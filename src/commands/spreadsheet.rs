use yard::{parser, evaluator};
use std::{
    error::Error,
    io::{self,{BufReader,Empty,Read,stdout},Write},
    borrow::Borrow,
    fs::{self,File},
    path::Path,
    sync::{Arc, Mutex},
    num::ParseIntError
};
use serde_json::{self,value::Value::Array};
use serde::{Serialize, Deserialize};
use csv::Writer;


#[derive(Debug)]
enum SpreadsheetError {
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    MutexError,
    IndexError,
    NotNumberError,
    ExitRequested,
}

const SPREADROW:usize = 12;
const SPREADCOL:usize = 5;

lazy_static!{
    static ref GRID: Mutex<Vec<Vec<Cell>>> = Mutex::new(vec![vec![Cell::Empty; SPREADROW]; SPREADCOL]);
}


pub(crate) fn enter_command(input: String) -> String{
    if 2 > input.len() {
        return format!("{}","Please enter a proper command");
    } else {
        match process_command(String::from(input)) {
            Ok(output) => output,
            //Err(SpreadsheetError::ExitRequested) => std::process::exit(1),
            Err(SpreadsheetError::IndexError) => format!("Index error, please try again"),
            Err(SpreadsheetError::MutexError) => format!("Try again"),
            Err(SpreadsheetError::ParseIntError(_e)) => format!("Try again"),
            Err(e) => format!("{:#?}", e),
        }
    }
}

#[derive(Debug, Clone,Deserialize,Serialize)]
enum Cell {
    Text(String),
    Number(f64),
    Formula(FormulaCell),
    Empty,
}
impl Cell{
    pub fn compare(&self, other: Cell)->u8{
        2
    }
    pub fn cell_text(&self) -> String {
        match self {
            Cell::Text(string) =>{cell_text_spaces(string.to_string().borrow())},
            Cell::Number(number) =>{cell_text_spaces(number.to_string().borrow())},
            Cell::Formula(_form) => {cell_text_spaces(&self.get_value().unwrap_or(0.0).to_string())},
            Cell::Empty => " ".repeat(10).to_string(),
        }
    }

    pub fn full_text(&self) -> String {
        match self {
            Cell::Text(string) => format!("\"{}\"", string.clone()),
            Cell::Number(number) => number.to_string(),
            Cell::Formula(string)=> string.get_text(),
            Cell::Empty => String::from(" "),
        }
    }
    pub fn base_text(&self)-> String{
        match self {
            Cell::Text(string) =>format!("\"{}\"", string.clone()),
            Cell::Number(number) =>number.to_string(),
            Cell::Formula(_formula_cell) => {self.get_value().unwrap_or(0.0).to_string().to_string()},
            Cell::Empty => " ".to_string(),
        }
    }
    pub fn get_value(&self) -> Option<f64> {
        match self {
            Cell::Number(number) => Some(*number),
            Cell::Formula(formula) => Some(match formula.string_to_f64(){
                Ok(out) => out,
                Err(_e) => 0.0,
            }),
            Cell::Text(_) => None,
            Cell::Empty => None,
        }
    }
}
#[derive(Debug, Clone,Deserialize,Serialize)]
struct FormulaCell{
    //cell: String,
    command: String,
}
impl FormulaCell{
    pub fn new(input: String)->FormulaCell{
        self::FormulaCell{
            command:input,
        }
    }
    pub fn string_to_f64(&self)-> Result<f64,SpreadsheetError>{
        let input = &self.command[1..self.command.len()-1];
        let mut input_arr:Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();
        for i in 0..input_arr.len(){
            if input_arr[i].len() == 2 as usize && &input_arr[i].to_uppercase().as_bytes()[0] >= &65 {
                let row: u8 = input_arr[i].to_uppercase().as_bytes()[0] - 65;
                let col: u8 = match input_arr[i].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                    Ok(output) => output,
                    Err(_e) => 0,
                };
                {
                    let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                    let spreadsheet = db.clone();
                    std::mem::drop(db);
                    input_arr[i] = match spreadsheet[row as usize][(col - 1) as usize].get_value(){
                        Some(num)=> num.to_string(),
                        None=> "0".to_string(),

                    };
                }
            }
        }
        match input_arr[0].to_uppercase().as_ref(){
            "SUM"=>{
                if input_arr.len() >= 2 {
                    let input_loc:Vec<String> = input_arr[1]
                        .split("-").map(|x| x.to_string()).collect();
                    let (start_row, start_col): (u8,u8) = (input_loc[0].to_uppercase().as_bytes()[0] - 65,
                       match input_loc[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                           Ok(output) => output,
                           Err(_e) => 0,
                       });
                    let (end_row, end_col): (u8,u8) = (input_loc[1].to_uppercase().as_bytes()[0] - 65,
                       match input_loc[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                           Ok(output) => output,
                           Err(_e) => 0,
                       });
                    let mut var: f64 = 0.0;
                    for c in start_col..=end_col {
                        for r in start_row..=end_row{
                            println!("{} {}",c,r);
                            {
                                let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                                let spreadsheet = db.clone();
                                std::mem::drop(db);
                                var += match spreadsheet[c as usize][r as usize].get_value() {
                                    Some(t) => t,
                                    None => 0.0,
                                } as f64;
                            }
                        }
                    }
                    println!("{}",var);
                    return Ok(var)
                }
                return Ok(0.0)
            }
            "AVG"=>{
                if input_arr.len() >= 2 {
                    let input_loc:Vec<String> = input_arr[1]
                        .split("-").map(|x| x.to_string()).collect();
                    let (start_row, start_col): (u8,u8) = (input_loc[0].to_uppercase().as_bytes()[0] - 65,
                       match input_loc[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                           Ok(output) => output,
                           Err(_e) => 0,
                       });
                    let (end_row, end_col): (u8,u8) = (input_loc[1].to_uppercase().as_bytes()[0] - 65,
                       match input_loc[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                           Ok(output) => output,
                           Err(_e) => 0,
                       });
                    let mut var: f64 = 0.0;
                    let mut times = 0.0;

                    for c in start_col..=end_col {
                        for r in start_row..=end_row{
                            times += 1.0;
                            {
                                let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                                let spreadsheet = db.clone();
                                std::mem::drop(db);
                                var += match spreadsheet[c as usize][r as usize].get_value() {
                                    Some(t) => t,
                                    None => 0.0,
                                } as f64;
                            }
                        }
                    }
                    println!("{}",var/times);
                    return Ok(var/times)
                }

                return Ok(0.0)
            }
            _=>{
                if let Ok(tokens) = parser::parse(&input_arr.join(" ")){
                    let result = evaluator::eval(&tokens);
                    return Ok(result as f64);
                }
                return Ok(0.0)
            }
        }
    }
    pub fn get_text(&self) -> String{ (&self.command).parse().unwrap() }
}
fn get_grid_text() -> Result<String,SpreadsheetError> {
    let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
    let row: u8 = SPREADROW as u8;
    let col: u8 = SPREADCOL as u8;
    let start: u8 = 65;
    let mut top = String::from("   |");
    let mut bottom = String::new();
    let grid = db.clone();
    std::mem::drop(db);
    for i in 0..col {
        top.push_str(&format!("{}{}|", (start + i) as char, " ".repeat(9)));
    }
    for i in 0..row {
        if i >= 9 {
            bottom.push_str(&format!("{} |", i + 1));
        } else {
            bottom.push_str(&format!("{}  |", i + 1));
        }
        for a in 0..col {

            bottom.push_str(&format!("{}|", grid[a as usize][i as usize].cell_text()));
        }
        bottom.push('\n');
    }
    Ok(format!("{}\n{}", top, bottom))

}
fn cell_text_spaces(string: &String) -> String {
    let mut spaces = String::new();
    for _i in 0..10 - (string.len() as i32) {
        spaces.push(' ');
    }

    if string.len() > 10 {
        string[0..10].parse().unwrap()
    } else {
        format!("{}{}", &string, spaces)
    }

}
fn process_command(input:String) -> Result<String,SpreadsheetError>{
    let input: Vec<&str> = input.splitn(3,' ').collect();
    match input[0].to_uppercase().as_ref() {
        "LOAD"=>{
            let serialized = fs::read_to_string("spreadsheet.txt")
                .expect("Something went wrong reading the file");
            let b:Vec<Vec<Cell>> = serde_json::from_str(&serialized).unwrap();
            let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
            let row: usize = {
                if db.len() < b.len(){
                    db.len()
                }else{
                    b.len()
                }
            };
            let col:usize ={
                if SPREADCOL < db[0].len(){
                    SPREADCOL
                }else{
                    db[0].len()
                }
            };
            for r in 0..row{
                for c in 0..col{
                    db[r as usize][c as usize] = b[r as usize][c as usize].clone();
                }
            }
            Ok(String::from("Loaded sheet"))
        }
        "EXPORT"=>{
            let wtr = Writer::from_path("export.csv");
            let mut key = wtr.unwrap();
            let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
            let a = db.clone();
            for r in 0..a.len(){
                let mut arr = vec![String::new()];
                for c in 0..a[r].len(){
                    arr.insert(c,a[c as usize][r as usize].cell_text());
                }
                if let Err(e) =key.write_record(&arr){
                  println!("error writing to record {}",e);
                };
            }
            std::mem::drop(a);
            Ok(String::from("Spreadsheet exported as *export.csv*, use command *export* to get a copy"))
        }
        "SAVE"=>{
            let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
            let a = db.clone();
            let path = Path::new("spreadsheet.txt");
            let display = path.display();
            let mut file = match File::create(&path){
                Err(why) => panic!("couldn't create {}: {}", display, why.description()),
                Ok(file)=> file,
            };
            let serialized = serde_json::to_string(&a).unwrap();
            match file.write_all(serialized.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
                Ok(_) => println!("successfully wrote to {}", display),
            }

            println!("{}",serialized);
            std::mem::drop(a);
            Ok(String::from("Spreadsheet saved"))
        }
        "SORTA"=>{
            Ok(String::from("Command SORTA entered"))
        }
        "PRINT"|"SPREADSHEET"|"SPREAD"=>{
            Ok(get_grid_text().expect(""))
        }
        "SORTD"=>{
            Ok(String::from("Command SORTD entered"))
        }
        "CLEAR"=> {
            if input.len() == 2 {
                let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                let row: u8 = input[1].to_uppercase().as_bytes()[0] - 65;
                let col: u8 = match input[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                    Ok(output) => output - 1,
                    Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                };
                if col <= db.len() as u8{
                    db[row as usize][col as usize] = Cell::Empty;
                }else{
                    return Err(SpreadsheetError::IndexError);
                }

            } else {
                {
                    let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                    let row = SPREADROW;
                    let col = SPREADCOL;
                    for r in 0..row {
                        for c in 0..col {
                            db[c][r] = Cell::Empty;
                        }
                    }
                }
            }
            return Ok(get_grid_text().expect(""));
        }
        "MEM"=>{
            if input.len() >= 2{
                let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                let row: u8 = input[1].to_uppercase().as_bytes()[0] - 65;
                let col: u8 = match input[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                    Ok(output) => output - 1,
                    Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                };
                if col <= db.len() as u8{
                    Ok(String::from(format!("{:p}",&db[row as usize][col as usize])))
                }else{
                    Err(SpreadsheetError::IndexError)
                }
            }else{
                {
                    let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                    Ok(String::from(format!("{:p}", &db)))
                }
            }
        }
        _ =>{
            {
                if input.len() >= 3 {
                    let mut input_two = input[2];
                    if &input_two[0..1] == "(" && &input_two[input_two.len() - 1..input_two.len()] == ")" {
                        //lock is called on db grid here, causing error when lock is called by FormulaCell creation. TODO: Sanitize with values here
//                        let input = &input_two[0..input_two.len()];
//                        let mut input_arr:Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();
//                        for i in 0..input_arr.len(){
//                            if input_arr[i].len() == 2 as usize && &input_arr[i].to_uppercase().as_bytes()[0] >= &65 {
//                                let row: u8 = input_arr[i].to_uppercase().as_bytes()[0] - 65;
//                                let col: u8 = match input_arr[i].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
//                                    Ok(output) => output,
//                                    Err(_e) => 0,
//                                };
//                                    input_arr[i] = match db[row as usize][(col - 1) as usize].get_value(){
//                                        Some(num)=> num.to_string(),
//                                        None=> "0".to_string(),
//
//                                    };
//                            }
//                        }
//                        let passedval = &input_arr.clone().join(" ");
                        let form_val = Cell::Formula(FormulaCell::new(input_two.parse().unwrap()));
                        let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                        let row: u8 = input[0].to_uppercase().as_bytes()[0] - 65;
                        let col: u8 = match input[0].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                            Ok(output) => output - 1,
                            Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                        };
                        if col > db.len() as u8{
                            return Err(SpreadsheetError::IndexError);
                        }else{
                            db[row as usize][col as usize] = form_val.clone();
                        }
                    } else if &input_two[0..1] == "\"" && &input_two[input_two.len() - 1..input_two.len()] == "\"" {
                        {
                            let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                            let row: u8 = input[0].to_uppercase().as_bytes()[0] - 65;
                            let col: u8 = match input[0].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                                Ok(output) => output - 1,
                                Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                            };
                            if col > db.len() as u8{
                                return Err(SpreadsheetError::IndexError);
                            }else{
                                db[row as usize][col as usize] = Cell::Text(input_two[1..input_two.len() - 1].to_string());
                            }
                        }
                    } else {
                        {
                            let mut db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                            let row: u8 = input[0].to_uppercase().as_bytes()[0] - 65;
                            let col: u8 = match input[0].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                                Ok(output) => output - 1,
                                Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                            };
                            if col > db.len() as u8{
                                return Err(SpreadsheetError::IndexError);
                            }
                            if &input_two[input_two.len() -1..input_two.len() ] == "%"{
                                db[row as usize][col as usize] = Cell::Number(match input_two[0..input_two.len()-1].parse::<f64>() {
                                    Ok(num) => num/ 100 as f64,
                                    Err(e) => return Err(SpreadsheetError::ParseFloatError(e))
                                });
                            }else{
                                db[row as usize][col as usize] = Cell::Number(match input_two.parse::<f64>() {
                                    Ok(num) => num,
                                    Err(e) => return Err(SpreadsheetError::ParseFloatError(e))
                                });
                            }
                        }
                    }
                } else {
                    {
                        let db = GRID.lock().map_err(|_| SpreadsheetError::MutexError)?;
                        let row: u8 = input[0].to_uppercase().as_bytes()[0] - 65;
                        let col: u8 = match input[0].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                            Ok(output) => output - 1,
                            Err(e) => return Err(SpreadsheetError::ParseIntError(e)),
                        };
                        if col > db.len() as u8{
                            return Err(SpreadsheetError::IndexError);
                        }
                        return Ok("cell value: ".to_owned() + &db[row as usize][col as usize].full_text());
                    }
                }
            }
            return Ok(get_grid_text().expect(""));
        }
    }
}
