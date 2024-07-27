use playout::PlayoutModule;

fn main() {
    let playout_str = include_str!("./example.playout");

    match PlayoutModule::try_from(playout_str) {
        Ok(module) => {
            let mut writer = String::new();
            module.show(&mut writer);
            println!("{}", writer)
        }
        Err(e) => {
            println!("Error: {} {}", e.to_string(), e.span().start().line);
        }
    };

    use ash::vk;
    let out = playout_macro::layout!("./example.playout", 3);
    println!("{:#?}", out);
}
