use playout::SetLayout;

fn main() {
    let playout_str = include_str!("./example.playout");

    match SetLayout::try_from(playout_str) {
        Ok(set_layout) => {
            dbg!(set_layout);
        }
        Err(e) => {
            println!("Error: {}", e.to_string());
        }
    }
}
