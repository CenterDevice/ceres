sub_module!("images", "Do infrastructure stuff with images", list, build);

#[derive(Debug, Serialize)]
pub struct Resource {
    pub project: String,
    pub name: String,
}

