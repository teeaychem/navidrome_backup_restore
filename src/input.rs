use std::collections::BTreeSet;

/// get n such that low ≤ n < high
pub fn get_number_in_range(msg: &str, low: usize, high: usize) -> usize {
    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        println!("{msg}");
        match stdin.read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Something went wrong reading the input.
{e:?}"
                );
                std::process::exit(9);
            }
        };
        match input.trim_end().parse::<usize>() {
            Ok(n) => {
                if low <= n && n <= high {
                    return n;
                } else {
                    println!("It is not the case that {low} ≤ {n} ≤ {high}, let's go again!");
                    input.clear();
                }
            }
            Err(e) => {
                println!(
                    "Failed to get number n such that {low} ≤ n ≤ {high} with error:
\t{e:?}
So, let's go again!"
                );
                input.clear();
            }
        }
    }
}

pub fn get_first_char(options: &[&str]) -> char {
    let stdin = std::io::stdin();
    let mut input = String::new();

    println!("What would you like to do?");
    let mut ok_chars: BTreeSet<char> = BTreeSet::default();
    println!();
    for x in options {
        let (a, b) = x.split_at(1);
        println!("{:-8}[{}]{}", " ", &a, b);
        ok_chars.insert(a.chars().next().unwrap());
    }
    println!();

    loop {
        match stdin.read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Something went wrong reading the input.
{e:?}"
                );
                std::process::exit(9);
            }
        };
        let input_char = input.trim().chars().next();

        match input_char {
            Some(c) => {
                if ok_chars.contains(&c) {
                    return c;
                }
            }
            _ => {
                println!("You entered: \"{input}\", … let us try again…");
            }
        }
        input.clear();
        println!("character for choice");
    }
}
