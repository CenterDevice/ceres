sub_module!("consul", "Do stuff of consul", list);

use std::str::FromStr;

#[derive(Debug)]
pub enum NodeField {
    Id,
    Name,
    MetaData(Option<Vec<String>>),
    Address,
    ServicePort,
    ServiceTags,
    ServiceId,
    ServiceName,
    Healthy,
}


impl FromStr for NodeField {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "Id" => Ok(NodeField::Id),
            "Name" => Ok(NodeField::Name),
            s if s.starts_with("MetaData") => {
                let filter = extract_metadata_filter(s);
                Ok(NodeField::MetaData(filter))
            },
            "Address" => Ok(NodeField::Address),
            "ServicePort" => Ok(NodeField::ServicePort),
            "ServiceTags" => Ok(NodeField::ServiceTags),
            "ServiceId" => Ok(NodeField::ServiceTags),
            "ServiceName" => Ok(NodeField::ServiceName),
            "Healthy" => Ok(NodeField::Healthy),
            _ => Err(Error::from_kind(ErrorKind::ModuleFailed(NAME.to_owned())))
        }
    }
}

fn extract_metadata_filter(metadata_str: &str) -> Option<Vec<String>> {
    if metadata_str.len() < 9 {
        return None;
    };
    let metadata = &metadata_str[9..]; // Safe because we call this function only when the prefix 'Metadata:' has been seen
    let metadata_filter: Vec<_> = metadata.split(':').map(String::from).collect();

    Some(metadata_filter)
}
