use serde_json;
use std::io;

#[derive(Debug, Deserialize)]
struct Instance {
    instance_id: String
}

fn read_instance_ids(args: &ArgMatches) -> Result<Vec<String>> {
    let instance_ids: Vec<_> = args.values_of("instance_ids")
        .unwrap() // Safe
        .map(String::from)
        .collect();

    // Let's check if we shall read instance ids from stdin
    if instance_ids.len() == 1 && instance_ids[0] == "-" {
        read_instance_ids_from_stdin()
    } else {
        Ok(instance_ids)
    }
}

fn read_instance_ids_from_stdin() -> Result<Vec<String>> {
    let instances: Vec<Instance> = serde_json::from_reader(io::stdin())
        .chain_err(|| ErrorKind::ModuleFailed("Cannot read instance ids from stdin".to_string()))?;

    let instance_ids: Vec<String> = instances.into_iter()
        .map(|i| i.instance_id)
        .collect();

    Ok(instance_ids)
}

sub_module!("instances", "Do stuff with instances", list, run, ssh, start, stop, terminate);

