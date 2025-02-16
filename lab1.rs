use std::io;
struct Computer{
    brand: String,
    processor: String,
    memory: u32,
}

impl Computer {
    fn new(brand: String, processor: String, memory: u32) -> Self {
        Self {
            brand: brand,
            processor: processor,
            memory: memory,
        }
    }
    fn display_computer(&self) {
        println!("Computer Details:");
        println!("Brand: {}", self.brand);
        println!("Processor: {}", self.processor);
        println!("Memory: {} GB", self.memory);
    }
}
fn main() {
    //ex 1
    println!("Gabi");

    //ex 2
    let a=10;
    let b=17;
    let temp;
    if a>b {temp=a;}
    else {temp=b;}
    println!("{temp}");
    
    //ex 3
    let m=15;
    let n=5;
    if m%n==0 {println!("{m} is divisible by {n}");}
    else {println!("{m} is not divisible by {n}");}
    
    //ex 4
    let arr=[4, 5, 5, 7, 9, 1, 2];
    let mut max=arr[0];
    for i in arr{
        if i>max {max=i;}
    }
    println!("The maximum value in the array is {max}");
    
    //ex 5
    let computer1=Computer::new(
        String::from("Asus"),
        String::from("amd ryzen 9"),
        16,
    );
    computer1.display_computer();
    
    //ex 6
    let computers=[
        Computer::new(String::from("Apple"), String::from("Apple M1"), 16),
        Computer::new(String::from("Dell"), String::from("Intel Core i7"), 32),
        Computer::new(String::from("HP"), String::from("AMD Ryzen 5"), 8),
    ];
    loop{
        println!("Menu:");
        println!("a. Print all computers");
        println!("b. Print the computer with the largest amount of memory");
        println!("Please enter your choice (a/b):");

        let mut choice=String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice=choice.trim().to_lowercase();

        match choice.as_str(){
            "a" => {
                for computer in &computers {
                    computer.display_computer();
                    println!();
                }
            }
            "b" => {
                if let Some(largest)=computers.iter().max_by_key(|c| c.memory) {
                    largest.display_computer();
                    println!();
                }
            }
            _ => {
                println!("Invalid option, exiting...");
                break;
            }
        }
    }
}
