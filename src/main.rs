extern crate xmlJSON;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate inflate;
extern crate percent_encoding;

use xmlJSON::XmlDocument;
use rustc_serialize::json::{ToJson, Json};
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use data_encoding::BASE64;

use inflate::inflate_bytes;
use std::str::from_utf8;
use percent_encoding::{percent_decode};

fn main() {
    let mut file = File::open("Odyssey.xml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    
    let doc: XmlDocument = XmlDocument::from_str(&contents[..]).unwrap();
    let json = doc.to_json();
    match json.find_path(&["mxfile", "diagram"]) {
        Some(array) => {
            array.as_array().unwrap().iter()
                .map(|item|{
                    item.as_object()
                        .and_then(|obj|obj.get("_"))
                        .and_then(|obj|obj.as_string()).unwrap()
                })
                .for_each(|base64_str| {
                    // let v = BASE64.decode(base64_str.as_bytes())
                    //     .and_then(|code| inflate_bytes(&code))
                    //     .and_then(|inflated| from_utf8(&inflated))
                    //     .and_then(|url_encoded| percent_decode(url_encoded.as_bytes()).decode_utf8())
                    //     .map(|decoded| decoded.into_owned()) //Cow
                    //     .and_then(|data_xml| XmlDocument::from_str(&data_xml)).ok()
                    //     .and_then(|doc|doc.to_json())
                    //     .and_then(|json| json.find_path(&["mxGraphModel", "root", "object"]))
                    //     .and_then(|object_array| object_array.as_array())
                    //     .and_then(|array| array.iter().map(|json| {
                    //             json.as_object()
                    //                 .and_then(|obj| obj.get("$"))
                    //         })
                    //         .filter(|json|{
                    //             match json.map(|o|o.as_object()).map(|m|m.unwrap().contains_key("tooltip")) {
                    //                 Some(true) => true,
                    //                 _ => false,
                    //             }
                    //         })
                    //         .for_each(|with_tooltip| {
                    //             let element = with_tooltip.unwrap().as_object().unwrap();
                    //             println!("{:?} -> {:?}", element.get("label"), element.get("tooltip"));
                    //         })
                    //     );
                    // ;
                    // println!("{:?}", v);
                    let code = BASE64.decode(base64_str.as_bytes()).unwrap();
                    let inflated = inflate_bytes(&code).unwrap();
                    let url_encoded = from_utf8(&inflated).unwrap();
                    let data_xml = percent_decode(url_encoded.as_bytes()).decode_utf8().unwrap().into_owned();
                    let diagram_json: Json = XmlDocument::from_str(&data_xml).unwrap().to_json();
                    
                    match diagram_json.find_path(&["mxGraphModel", "root", "object"]) {
                        Some(object_array) => {
                            object_array.as_array().unwrap().iter()
                                .map(|json| json.as_object().unwrap().get("$"))
                                .filter(|json|{
                                    match json.map(|o|o.as_object()).map(|m|m.unwrap().contains_key("tooltip")) {
                                        Some(true) => true,
                                        _ => false,
                                    }
                                })
                                .for_each(|with_tooltip| {
                                    let element = with_tooltip.unwrap().as_object().unwrap();
                                    println!("{:?} -> {:?}", element.get("label"), element.get("tooltip"));
                                });
                        },
                        _ => (),
                    }
                    //println!("type: {:?}", diagram_json.as_object().unwrap().get("mxGraphModel").unwrap());
                });
        },
        None => println!("No diagram found"),
    };
}
