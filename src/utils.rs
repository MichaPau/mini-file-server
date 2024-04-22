use std::fs;
use std::path::PathBuf;

use rocket::fs::FileName;
use rocket::fs::TempFile;
use rocket::serde::Serialize;
use crate::AppState;

type GenericError = Box<dyn std::error::Error + Send + Sync + 
'static>;
type GenericResult<T> = Result<T, GenericError>;

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
pub fn create_breadcrump_items(p: &PathBuf) -> Vec<String> {
    
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
    let mut dir:PathBuf = PathBuf::from("./static/data/");
    dir.push(&folder);
    let root_dir = fs::read_dir(&dir)?;
    let paths_vec:Vec<FileItem> = root_dir.filter_map(Result::ok).map(|item| {
           //let mut icon_path_prefix:PathBuf = PathBuf::from("/assets/icons/");
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
           
            let p = item.file_name();
            //let name = p.clone().into_string().unwrap();
            let mut path:PathBuf = PathBuf::new();
            path.push(&dir);
            path.push(&p);
            
            let mut url_path: PathBuf = [std::path::MAIN_SEPARATOR_STR, "data"].iter().collect();
            // let mut file_path = PathBuf::from("data");
            // file_path.push("data");
            url_path.push(&folder);
            url_path.push(item.file_name());
            let file_path = format!("{}", url_path.display());
            if !is_folder {
                url_path.push("?download");
            }
            let path_str = format!("{}", url_path.display());
            
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
                url_path: path_str,
                file_path
                //file_path: n2,
            }

        }).collect();

        Ok(paths_vec)
}