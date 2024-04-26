#[macro_use] extern crate rocket;

//use std::io;
use std::path::{Path, PathBuf};

use rocket::form::Form;

use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::tokio::fs;
use rocket::{Request, Response, State};
use rocket::fs::{FileServer, NamedFile};
//use rocket::config::Config;
//use rocket::fs::relative;
//use rocket::serde::Serialize;
use rocket::serde::json::{serde_json, Value};

//use rocket_dyn_templates::Template;
use rocket_dyn_templates::{Template, context};
use utils::{get_files, Folder, ServerError};
use utils::create_breadcrump_items;
use utils::make_file_name;
use utils::Upload;


mod utils;
struct AppState {
    
    icon_set: String,
    available_icons: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let catalog = include_str!("../static/assets/icons/classic/catalog.json");
        let v: Value = serde_json::from_str(catalog).unwrap();
        
        let available_icons = v.as_array().unwrap().iter().map(|item| item.as_str().unwrap().to_owned()).collect();
        
        Self {
            icon_set: "classic".to_string(),
            available_icons,
        }
    }
}

struct DownloadFile(NamedFile);

impl<'r> Responder<'r, 'static> for DownloadFile {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Content-Disposition", "attachement")
            .ok()
    }
}

#[get("/test")]
fn test() -> Result<String, ServerError> {
    println!("Test route..");
    //Ok("This is the test result".into())
    Err(ServerError::GeneralError("Error string".to_string()))
}


#[get("/")]
fn index(app_state: &State<AppState>) -> Template {
    println!("in index route");
    match get_files(app_state, &PathBuf::new()) {
        Ok(data) => {
            Template::render("base", context! {
                title: "Mini File Server",
                paths: vec!["/"],
                folder: "",
                data
            })
        
        },
        Err(e) => {
            Template::render("error", context! {
                error: e.to_string()
            })
        },
    } 
}

#[get("/data/<file_path..>?download")]
async fn file_content(file_path: PathBuf) -> Option<DownloadFile> {
    println!("in /(file-content):{:?}", file_path);
    NamedFile::open(Path::new("static/data/").join(file_path)).await.ok().map(|nf| DownloadFile(nf))
    
}
#[put("/data/<file_path..>")]
fn get_folder_items(app_state: &State<AppState>, file_path: PathBuf) -> Template {
    println!("got put request:{:?}", file_path);
   
    match get_files(app_state, &file_path) {
        Ok(data) => {
            Template::render("folder_result", context! {
                data,
            })
        
        },
        Err(e) => {
            Template::render("error", context! {
                error: e.to_string()
            })
        },
    }
    
}


#[get("/data/<file_path..>")]
fn get_folder_items_page(app_state: &State<AppState>, file_path: PathBuf) -> Template {
    println!("got get request:{:?}", file_path);
    //format!("put request {}", file_path);
    let breadcrump = create_breadcrump_items(&file_path);
    match get_files(app_state, &file_path) {
        Ok(data) => {
            Template::render("base", context! {
                title: "Mini File Server",
                paths: breadcrump,
                folder: file_path,
                data
            })
        
        },
        Err(e) => {
            Template::render("error", context! {
                error: e.to_string()
            })
        },
    }
    
}

#[post("/create_dir", data = "<new_folder>")]
async fn create_dir(app_state: &State<AppState>, new_folder: Form<Folder>) -> Result<Template, ServerError> {
    if !new_folder.name.chars().all(|x| x.is_alphanumeric() || x == '_') {
        return Err(ServerError::GeneralError(format!("name is not valid: {}", &new_folder.name)));
    }
    let mut dir_path = PathBuf::from("./static/data/");
    dir_path.push(&new_folder.path);

    if !dir_path.exists() {
        return Err(ServerError::GeneralError(format!("path is not valid: {}", &new_folder.path)));
    }
    dir_path.push(&new_folder.name);
    println!("create new folder in: {:?}", dir_path);

    match std::fs::create_dir(dir_path) {
        Ok(()) => {
           Ok(get_folder_items(&app_state, PathBuf::from(&new_folder.path)))
        },
        Err(e) => Err(ServerError::GeneralError(format!("Error: {}", e)))
    }

}
#[post("/upload", data = "<upload>")]
async fn upload_post(app_state: &State<AppState>, upload: Form<Upload<'_>>) -> Result<Template, ServerError> {
    
    let mut path = PathBuf::from("./static/data/");
    let folder = String::from(&upload.folder);
    println!("upload post: {}", folder);
    path.push(String::from(&upload.folder));

    if !path.exists() {
        return Err(ServerError::GeneralError(format!("path is not valid: {:?}", path)));
    }

    let mut files = upload.into_inner().files;
    
    let mut upload_result: Vec<String> = vec![];
    for (_i, file) in files.iter_mut().enumerate() {
        let mut file_path = PathBuf::from(&path);

        if let Ok(file_name) = make_file_name(file.raw_name()) {
            file_path.push(&file_name);
            // file_path.push(file.name().unwrap());
            println!("try to store file in path: {:?}", file_path);
            if !file_path.exists() {
                match file.persist_to(&file_path).await {
                    Ok(()) => upload_result.push(format!("{} saved.", &file_name)),
                    Err(e) => upload_result.push(format!("Error saving {} [{}]", file_name, e)),

                }
            } else {
                upload_result.push(format!("File name exists in folder {}", file_name));
            }
        } else {
            upload_result.push(format!("File name error with: {:?}", file.raw_name()));
        }
    }

    println!("{:?}", upload_result);
   
    //let p :PathBuf = [std::path::MAIN_SEPARATOR_STR, "data", &folder].iter().collect();
    let p :PathBuf = PathBuf::from(&folder);
    Ok(get_folder_items(&app_state, p))
    // let uri = uri!(get_folder_items_page(p)).to_string();
    // Redirect::to(uri)
}

#[delete("/delete/data/<file_path..>")]
async fn delete_item(file_path: PathBuf) -> Result<String, ServerError> {
    
    let mut path = PathBuf::from("./static/data/");
    path.push(&file_path);
   
    if path.is_dir() {
        match fs::remove_dir_all(&path).await {
            Ok(()) => Ok(format!("Folder {:?} deleted", file_path)),
            Err(e) => Err(ServerError::GeneralError(format!("Error: {:?}", e))),
        }
    } else {
        match fs::remove_file(&path).await {
            Ok(()) => Ok(format!("Item {:?} deleted", file_path)),
            Err(e) => Err(ServerError::GeneralError(format!("Error: {:?}", e))),
        }
    }
}

#[catch(default)]
fn default_catcher(status: Status, request: &Request) -> String {
    format!("Status: {}; Sorry, '{}' is not a valid path.", status.to_string(), request.uri())
}
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
   
    // let host: String = Config::figment().extract_inner("address").unwrap();
    // let port: u32 = Config::figment().extract_inner("port").unwrap();
    // let base_url = format!("http://{}:{}", host, port);
    // println!("{:?}", base_url);
    let _rocket = rocket::build()
        .manage(AppState::default())
        .mount("/", routes![test, index, get_folder_items, file_content, get_folder_items_page, upload_post, delete_item, create_dir])
        .mount("/scripts", FileServer::from("./static/scripts"))
        .mount("/styles", FileServer::from("./static/css"))
        .mount("/assets", FileServer::from("./static/assets"))
        .attach(Template::fairing())
        .register("/", catchers![default_catcher])
        .launch()
        .await?;

    Ok(())
}