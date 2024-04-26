async function test_fetch() {
    console.log("test_fetch");
    await fetch("/test").then((reponse) => {
        if(reponse.ok) {
            reponse.text().then((text) => {
                console.log("text (ok): ", text);
            });
        } else if (reponse.status = 500) {
            reponse.text().then((text) => {
                console.log("error (err): ", text);
            });
        }
        
    }).catch((e) => {
        console.log("test error:", e);
    });
}

async function delete_item(item_path, elem_id) {
    console.log("delete item: ", item_path);
    if (window.confirm("Delete item:"+item_path)) {
        let url = item_path;
        await fetch(url, {method: "DELETE"})
            .then((reponse) => {
                if(reponse.ok) {
                    console.log("Item deleted..");
                    document.getElementById(elem_id).remove();
                } else {
                    let f = document.getElementById("feedback-container");
                    reponse.text().then((text) => {
                        console.log("Error: ", text);
                        f.innerText = text;
                        f.classList.add("show");
                    });
                }
            }
        );
    }
}

async function create_folder() {
    console.log("create_folder");
    let form = document.getElementById("create_folder_form");
    let container = document.getElementById("file-container");
    let formData = new FormData(form);

    await fetch("/create_dir", {
        method: "POST",
        body: formData,
    })
    .then((response) => {
        if(response.ok) {
            response.text().then((template) => container.innerHTML = template);
            form.reset();
        } else {
            let f = document.getElementById("feedback-container");
            response.text().then((error) => {
                console.log("Error: ", error);
                f.innerText = error;
                f.classList.add("show");
            });
        }
    })
}

async function upload() {
    console.log("upload");
    let form = document.getElementById("upload_form");
    let container = document.getElementById("file-container");
    let formData = new FormData(form);

    await fetch("/upload", {
        method: "POST",
        body: formData,
    })
    .then((response) => {
        if(response.ok) {
            response.text().then((template) => container.innerHTML = template);
            form.reset();
        } else {
            let f = document.getElementById("feedback-container");
            response.text().then((error) => {
                console.log("Error: ", error);
                f.innerText = error;
                f.classList.add("show");
            });
        }
    })
}