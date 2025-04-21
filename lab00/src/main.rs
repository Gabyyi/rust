//
fn greet(name: &str){
    println!("Hello {}",name);
}

//
fn odd_numbers(n: i32){
    let mut i=1;
    let mut k: i32=0;
    while k<n{
        println!("{}",i);
        i+=2;
        k+=1;
    }
}

//
fn first_even(slice: &[i32]) -> Option<i32>{
    for &i in slice{
        if i%2==0{
            return Some(i);
        }
    }
    None
}

//
fn first_long_string(strings: Vec<String>) -> Option<String>{
    for s in strings{
        if s.len()>4{
            return Some(s);
        }
    }
    None
}

//
enum Currency{
    Ron,
    Dollar,
    Euro,
    Pound,
    Bitcoin,
}
struct Transaction{
    amount: f64,
    currency: Currency,
}
fn value_in_ron(t: &Transaction) -> f64{
    match t.currency{
        Currency::Ron => t.amount,
        Currency::Dollar => t.amount*4.5,
        Currency::Euro => t.amount*5.0,
        Currency::Pound => t.amount*6.0,
        Currency::Bitcoin => t.amount*100000.0,
    }
}
fn total_value_in_ron(t: Vec<Transaction>) -> f64{
    let mut total=0.0;
    for transaction in t{
        total+=value_in_ron(&transaction);
    }
    total
}

//
#[derive(Debug)]
enum MyError{
    EmptyString,
    InvalidCharacter{position: usize, character: char},
    NegativeNumber,
}
fn parse_number(s: &str) -> Result<i32, MyError>{
    if s.len()==0{
        return Err(MyError::EmptyString);
    }
    if s.starts_with('-'){
        return Err(MyError::NegativeNumber);
    }
    for(i,c) in s.chars().enumerate(){
        if !c.is_digit(10){
            return Err(MyError::InvalidCharacter{position: i, character: c});
        }
    }
    match s.parse::<i32>(){
        Ok(n) if n>=0 => Ok(n as i32),
        Ok(_) => Err(MyError::NegativeNumber),
        Err(_) => Err(MyError::InvalidCharacter{position: 0,character: ' '}),
    }
}

//
// struct Complex{
//     real: f64,
//     img: f64,
// }
// impl Complex{
//     fn new(real: f64, img: f64) -> Self{
//         Complex{real, img}
//     }
//     fn abs(&self) -> f64{
//         (self.real*self.real+self.img*self.img).sqrt()
//     }
//     fn multiply(&self, c: &Complex) -> Complex{
//         Complex{real: self.real*c.real-self.img*c.img, img: self.real*c.img+self.img*c.real}
//     }
// }
fn main(){
    println!("Exercise 1:");
    greet("Gabi");

    println!("Exercise 2:");
    odd_numbers(10);

    println!("Exercise 3:");
    let a=[1,2,3,4,5,6,7,8,9,10];
    match first_even(&a[4..8]){
        Some(i) => println!("First even number is {}",i),
        None => println!("No even number found"),
    }

    println!("Exercise 4:");
    let strings=vec!["hello".to_string(),"world".to_string(),"Gabi".to_string()];
    match first_long_string(strings){
        Some(s) => println!("First long string is {}",s),
        None => println!("No long string found"),
    }

    println!("Exercise 5:");
    let t=vec![
        Transaction{amount: 100.0, currency: Currency::Ron},
        Transaction{amount: 100.0, currency: Currency::Dollar},
        Transaction{amount: 100.0, currency: Currency::Euro},
        Transaction{amount: 100.0, currency: Currency::Pound},
        Transaction{amount: 100.0, currency: Currency::Bitcoin},
    ];
    let total=total_value_in_ron(t);
    println!("Total value in Ron is {}",total);

    println!("Exercise 6:");
    match parse_number("i100"){
        Ok(value) => println!("Number is {}",value),
        Err(e) => println!("Error: {:?}",e),
    }

    println!("Exercise 7:");
}