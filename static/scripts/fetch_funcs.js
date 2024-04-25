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
        })
}