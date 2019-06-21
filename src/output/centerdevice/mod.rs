use std::collections::HashMap;

pub mod collections;
pub mod search;
pub mod users;

fn map_user_id_to_name<'a, 'b: 'a>(
    user_map: Option<&'a HashMap<String, String>>,
    id: &'b str,
) -> &'a str {
    if let Some(map) = user_map {
        return map.get(id).map(|x| x.as_ref()).unwrap_or(id);
    }

    id
}
