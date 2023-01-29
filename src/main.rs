use std::path::Path;
use std::fs::File;

use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use config::Config;

fn main() {

    let path = Path::new("./components.svg");
    let path_display = path.display();

    let mut components_reader = match Reader::from_file(path) {
        Ok(v) => v,
        Err(why) => panic!("Couldn't read {} for reason {}", path_display, why),
    };
    components_reader.trim_text(true);

    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build().unwrap();

    let path = Path::new("./output.svg");
    let path_display = path.display();

    let out_file = match File::create(path) {
        Ok(v) => v,
        Err(why) => panic!("Couldn't write to {} for reason {}", path_display, why),
    };
    let writer = Writer::new(out_file);

    crate::run(components_reader, writer, settings);
}
