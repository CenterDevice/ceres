sub_module!("asp", "Do infrastructure stuff with ASPs", list);

#[derive(Debug, Serialize)]
pub struct Asp {
    pub project: String,
    pub resource: String,
}

