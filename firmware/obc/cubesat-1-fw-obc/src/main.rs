use indoc::indoc;

fn main() {
    let cubesat = indoc! {"
           ___________  
         /            /|
        /___________ / |
        |           |  |
        |           |  |
        | CubeSat-1 |  |
        |           | / 
        |___________|/  
    "};

    println!(
        "{}\n{}",
        "Hello world!",
        "I dream to be an OBC firmware for the CubeSat-1 project when I will grow up (^_^)"
    );

    println!("{}", cubesat);
}
