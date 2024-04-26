use std::fs;
use std::path::PathBuf;


use path_slash::PathBufExt;
use rocket::fs::FileName;
use rocket::fs::TempFile;
//use rocket::response;
use rocket::serde::Serialize;
use crate::AppState;

//use path_slash::PathBufExt as _;
use path_slash::PathExt as _;

#[derive(Responder)]
pub enum ServerError {
    #[response(status = 500)]
    GeneralError(String)
}

pub type GenericError = Box<dyn std::error::Error + Send + Sync + 
'static>;
pub type GenericResult<T> = Result<T, GenericError>;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileItem {
    pub name: String,
    pub icon_path: String,
    pub is_folder: bool,
    pub url_path: String,
    pub file_path: String,

}

#[derive(FromForm)]
pub struct Upload<'r> {
    pub files: Vec<TempFile<'r>>,
    pub folder: String,
}

#[derive(FromForm)]
pub struct Folder {
    pub name: String,
    pub path: String,
}
pub fn create_breadcrump_items(p: &PathBuf) -> Vec<String> {
    
    let mut v:Vec<_> = p.ancestors()
        .map(|item| { 
            //item.to_string_lossy().to_string()
            item.to_slash().unwrap().to_string()
        }).collect();
    v.reverse();
    let crump_items: Vec<_> = v.iter().enumerate().map(|(i, item)| {
        
        if i == 0 {
            format!("<a class=\"breadcrump-item\" href=\"/\">/</a>")
        } else if i == v.len() - 1 {
            let last = item.split('/').last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item-active\" href=\"#\">{}</a>", last)
        } else {
            let last = item.split('/').last().unwrap_or("unknwon");
            format!("<a class=\"breadcrump-item\" href=\"/data/{item}\">{}</a>", last)
        }
        

    }).collect();
    crump_items
}

pub fn make_file_name(option_name: Option<&FileName>) -> GenericResult<String> {
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

pub fn get_files(app_state: &AppState, folder: &PathBuf) -> GenericResult<Vec<FileItem>> {
    let mut dir_path:PathBuf = PathBuf::from("./static/data/");
    dir_path.push(&folder);
    let dir = dir_path.to_slash().unwrap().to_string();
    let root_dir = fs::read_dir(&dir)?;
    let paths_vec:Vec<FileItem> = root_dir.filter_map(Result::ok)
        .filter(|item| item.path().extension().is_some() || item.path().is_dir())
        .map(|item| {
           
            let mut icon_path_prefix: PathBuf = [std::path::MAIN_SEPARATOR_STR, "assets", "icons"].iter().collect();
            let set_path = format!("{}", &app_state.icon_set);
            icon_path_prefix.push(set_path);
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
                } else {
                    icon_path_prefix.push("blank.svg");
                }  
            } else {
                icon_path_prefix.push("folder.svg");
            }
           
            let name = item.file_name().to_string_lossy().to_string();
            // let mut path:PathBuf = PathBuf::new();
            // path.push(&dir);
            // path.push(&p);
            
            let mut url_path: PathBuf = [std::path::MAIN_SEPARATOR_STR, "data"].iter().collect();
           
            url_path.push(&folder);
            url_path.push(item.file_name());
            let file_path = url_path.to_slash().unwrap().to_string();
            if !is_folder {
                url_path.push("?download");
            }
            let path_str = url_path.to_slash().unwrap().to_string();
            
           

            FileItem {
                name,
                icon_path: icon_path_prefix.to_slash().unwrap().to_string(),
                is_folder,
                url_path: path_str,
                file_path
                //file_path: n2,
            }

        }).collect();

        Ok(paths_vec)
}

#[test]
#[ignore]
fn test_dir() {
    let dir:PathBuf = PathBuf::from("./static/data/");
    let root_dir:Vec<_> = fs::read_dir(&dir).unwrap().filter_map(Result::ok).filter(|item| !item.file_name().to_str().unwrap().starts_with('.')).collect();

    for item in root_dir {
        println!("{:?}", item);
    }
}

#[test]
#[ignore]
fn test_path_slash() {
    let mut p = PathBuf::from("./data/folder");
    p.push("other");
    p.push("file.txt");
    println!("{:?}", p);

    let p2 = p.to_slash().unwrap();
    println!("{:?}", p2);


}

#[test]
fn test_something() {

    let p = PathBuf::from("../some/file/t.txt");
    let c = fs::canonicalize(&p).unwrap();
    println!("{:?}", p);
    println!("{:?}", c);
    // let d = fs::read_dir("./static/data/").unwrap();
    // let _files: Vec<_> = d.filter_map(Result::ok)
    //     .filter(|item| item.path().extension().is_some())
    //     .inspect(|item| {
    //         println!("{:?}", item.file_name());
    //     })
    //    .collect();
}