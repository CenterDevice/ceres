sub_module!("asp", "Do infrastructure stuff with ASPs", list, build);

#[derive(Debug, Serialize)]
pub struct Asp {
    pub project: String,
    pub resource: String,
}

