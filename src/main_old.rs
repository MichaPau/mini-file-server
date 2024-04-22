#[macro_use] extern crate rocket;

use std::fs;

use std::path::Path;
use std::path::PathBuf;

use rocket::form::Form;
use rocket::fs::FileName;
use rocket::fs::NamedFile;
use rocket::fs::TempFile;
use rocket::http::Status;

//use rocket::tokio::fs::File;
use rocket::Request;
//use rocket::{fs::relative, fs::FileServer}, serde::Serialize};
use rocket::State;
use rocket::fs::FileServer;
//use rocket::fs::relative;
use rocket::serde::Serialize;
use rocket::serde::json::{serde_json, Value};
//use rocket_dyn_templates::Template;
use rocket_dyn_templates::{Template, context};

struct AppState {
    icon_set: String,
    available_icons: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let catalog = include_str!("../static/icons/classic/catalog.json");
        let v: Value = serde_json::from_str(catalog).unwrap();
        
        let available_icons = v.as_array().unwrap().iter().map(|item| item.as_str().unwrap().to_owned()).collect();
        
        Self {
            icon_set: "classic".to_string(),
            available_icons,
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct FileItem {
    name: String,
    icon_path: String,
    is_folder: bool,
    file_path: String,

}

#[derive(FromForm)]
struct Upload<'r> {
    files: Vec<TempFile<'r>>,
    folder: String,
}


type GenericError = Box<dyn std::error::Error + Send + Sync + 
'static>;
type GenericResult<T> = Result<T, GenericError>;

#[catch(default)]
fn default_catcher(status: Status, request: &Request) -> String {
    format!("Status: {}; Sorry, '{}' is not a valid path.", status.to_string(), request.uri())
}
fn create_breadcrump_items(p: &PathBuf) -> Vec<String> {
    
    let mut v:Vec<_> = p.ancestors()
        .map(|item| { item.to_string_lossy().to_string() }).collect();
    v.reverse();
    let crump_items: Vec<_> = v.iter().enumerate().map(|(i, item)| {
        
        if i == 0 {
            format!("<a class=\"breadcrump-item\" href=\"/\">/</a>")
        } else if i == v.len() - 1 {
            let last = item.split(std::path::MAIN_SEPARATOR_STR).last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item-active\" href=\"#\">{}</a>", last)
        } else {
            let last = item.split(std::path::MAIN_SEPARATOR_STR).last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item\" href=\"/data/{item}?is_folder=true\">{}</a>", last)
        }
        

    }).collect();
    crump_items
}
// fn extract_file_type(path_option: Option<&Path>) -> Option<ContentType> {
//     let path;
//     match path_option {
//         Some(p) => path = p,
//         None => return None,
//     }

//     let result;
//     match tree_magic_mini::from_filepath(path) {
//         Some(r) => result = r,
//         None => return None,
//     }

//     let mime_type;
//     match ContentType::parse_flexible(result) {
//         Some(m) => mime_type = m,
//         None => return None,
//     }

//     Some(mime_type)
// }

fn make_file_name(option_name: Option<&FileName>) -> GenericResult<String> {
    if let Some(file_name) = option_name {
        let raw:String = file_name.dangerous_unsafe_unsanitized_raw().to_string();
        if let Some(ext_index) = raw.rfind('.') {
            if let Some(sanitized) = file_name.as_str() {
                if let Some(ext) = raw.get(ext_index+1..) {
                    Ok(format!("{}.{}", sanitized, ext))
                } else {
                    Err(GenericError::from("Could not extraxt extension."))
                }
            } else {
                Err(GenericError::from("File name not sanitizeable."))
            }
        } else {
            Err(GenericError::from("Could not find extension position."))
        }
    } else {
        Err(GenericError::from("Could not extract file name."))
    }
}
#[post("/upload", data = "<upload>")]
async fn upload_post(upload: Form<Upload<'_>>) -> String {
    let mut path = PathBuf::from("./static/data/");
    path.push(String::from(&upload.folder));
    let mut files = upload.into_inner().files;
    
    //https://github.com/rwf2/Rocket/issues/1600
    //let mut files = upload.files;
    //let folder = &upload.folder;
    let mut upload_result = vec![];
    for (_i, file) in files.iter_mut().enumerate() {
        let mut file_path = PathBuf::from(&path);
        
        // println!("{:?}", file.name().unwrap());
        // println!("{:?}", file.raw_name().unwrap().dangerous_unsafe_unsanitized_raw());
        // println!("{:?}", file.path().unwrap());
        // println!("{:?}", file.content_type().unwrap());
        // println!("{:?}", make_file_name(file.raw_name()));

        if let Ok(file_name) = make_file_name(file.raw_name()) {
            file_path.push(&file_name);
            // file_path.push(file.name().unwrap());
            println!("try to store file in path: {:?}", file_path);
            match file.persist_to(&file_path).await {
                Ok(()) => upload_result.push(format!("{} saved.", &file_name)),
                Err(e) => upload_result.push(format!("Error saving {} [{}]", file_name, e)),

            }
        } else {
            upload_result.push(format!("File name error with: {:?}", file.raw_name()));
        }
        
        
    }

    format!("{:?}", upload_result)
}
#[get("/data/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    println!("in /(files):{:?}", file);
    NamedFile::open(Path::new("static/data/").join(file)).await.ok()
    
}
// #[get("/data/<file..>")]
// async fn files_test(file: PathBuf) -> String {
//     println!("in /(files):{:?}", file);
//     //NamedFile::open(Path::new("static/data/").join(file)).await.ok()
//     format!("{:?}", file)
    
// }

#[get("/data/<folder..>?is_folder=true")]
fn folder(app_state: &State<AppState>, folder: PathBuf) -> Template {
    println!("in /(folder):{:?}", folder);
    let p_items = create_breadcrump_items(&folder);
    
    match get_files(app_state, &folder) {
        Ok(data) => {
            Template::render("base", context! {
                title: "Mini File Server",
                paths: p_items,
                folder,
                data
            })
        },
        Err(e) => {
            Template::render("error", context! {
                error: e.to_string()
            })
        }
    }
}

#[get("/")]
fn index(app_state: &State<AppState>) -> Template {
    //let ctx = 
    println!("in / (index)");
    match get_files(app_state, &PathBuf::new()) {
        Ok(data) => {
            Template::render("base", context! {
                title: "Mini File Server",
                paths: vec!["/"],
                folder: "/",
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

fn get_files(app_state: &AppState, folder: &PathBuf) -> GenericResult<Vec<FileItem>> {
    let mut dir:PathBuf = PathBuf::from("./static/data/");
    dir.push(&folder);
    let root_dir = fs::read_dir(&dir)?;
    let paths_vec:Vec<FileItem> = root_dir.filter_map(Result::ok).map(|item| {
            let mut icon_path_prefix:PathBuf = PathBuf::from("/icons/");
            icon_path_prefix.push(&app_state.icon_set);
            let is_folder = item.path().is_dir();
            if !is_folder {
                if let Some(ext) = item.path().extension() {
                    let ext_str = String::from(ext.to_str().unwrap_or("default"));
                    if app_state.available_icons.contains(&ext_str) {
                        let file_str = format!("{}.svg", ext_str);
                        icon_path_prefix.push(file_str);
                        
                    } else {
                        icon_path_prefix.push("default.svg");
                    }
                }
            } else {
                icon_path_prefix.push("folder.svg");
            }
           
            let p = item.file_name();
            //let name = p.clone().into_string().unwrap();
            let mut path:PathBuf = PathBuf::new();
            path.push(&dir);
            path.push(&p);
            
            let mut file_path = PathBuf::from("/data");
            file_path.push(&folder);
            file_path.push(item.file_name());
            if is_folder {
                file_path.push("?is_folder=true");
            }
            let path_str = format!("{}", file_path.display());
            
            // let mut file_path = String::from("/data");
            // file_path.push_str(&folder);
            // file_path.push_str(item.file_name());
            // if is_folder {
            //     file_path.push_str("?is_folder=true");
            // }

            FileItem {
                name: p.to_string_lossy().to_string(),
                icon_path: String::from(icon_path_prefix.to_str().unwrap()),
                is_folder,
                file_path: path_str,
                //file_path: n2,
            }

        }).collect();

        Ok(paths_vec)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
   
    let _rocket = rocket::build()
        .manage(AppState::default())
        .mount("/", routes![index, folder, files, upload_post])
        .mount("/", FileServer::from("static"))
        .attach(Template::fairing())
        .register("/", catchers![default_catcher])
        .launch()
        .await?;

    Ok(())
}

#[test]

fn test_breadcrump() {
    let p = Path::new("foo/bar/baz/qux");
    let mut v:Vec<_> = p.ancestors()
        .map(|item| { item.to_string_lossy().to_string() }).collect();
    v.reverse();
    let crump_items: Vec<_> = v.iter().enumerate().map(|(i, item)| {
        
        if i == 0 {
            format!("<a class=\"breadcrump-item\" href=\"/\">/</a>")
        } else if i == v.len() - 1 {
            let last = item.split('/').last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item-active\" href=\"#\">{}</a>", last)
        } else {
            let last = item.split('/').last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item\" href=\"/data/{item}?is_folder=true\">{}</a>", last)
        }
        

    }).collect();
    for item in crump_items {
        println!("{}", item);
    }
    
}

#[test]
#[ignore]
fn paths() {
    let mut p1 = PathBuf::from("/foo/bar/baz/");
    p1.push("file.ext");
    let mut p2 = PathBuf::from("/foo/");
    p2.push("bar/baz/");
    //p2.push("baz/");
    p2.push("some.txt");
    let p3: PathBuf = ["foo/", "bar/", "some.txt"].iter().collect();

    
    println!("{:?}", p1);
    println!("{:?}", p2);
    println!("{:?}", p3);
    //println!("{:?}", p4);
}

