use csv::ReaderBuilder;
use csv::WriterBuilder;
use std::error::Error;

fn read_csv() -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .from_path("data.csv")?;
    
    for result in rdr.records() {
        match result {
            Ok(record) => println!("{:?}", record),
            Err(e) => eprintln!("Error reading record: {}", e),
        }
    }
    Ok(())
}

fn write_csv() -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b',')
        .from_path("output.csv")?;
    
    wtr.write_record(&["name", "age", "city"])?;
    wtr.write_record(&["Alice", "30", "New York"])?;
    wtr.write_record(&["Bob", "25", "Los Angeles"])?;
    
    wtr.flush()?;
    Ok(())
}
