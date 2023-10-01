use playout::SetLayout;

fn main() {
    let playout_str = include_str!("./example.playout");

    match SetLayout::try_from(playout_str) {
        Ok(set_layout) => {
            let mut writer = String::new();
            let declarations = set_layout.to_declarations(0);
            for decl in declarations {
                glsl::transpiler::glsl::show_declaration(&mut writer, &decl);
            }
            println!("{}", writer)
        }
        Err(e) => {
            println!("Error: {}", e.to_string());
        }
    }
}
