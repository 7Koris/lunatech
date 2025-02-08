use cpal::traits::{DeviceTrait, HostTrait};
use colored::Colorize;
use gag::Gag;

/// Repeatedly attempts to capture valid user input
fn prompt_user_input(patterns: &[String], prompt: String) -> String {
    loop {
        println!("{prompt}"); 
        let mut input = String::new();


        let result = std::io::stdin().read_line(&mut input);
        if result.is_err() {
            continue;
        }

        let input = input.trim();
        if patterns.contains(&input.to_owned()) {
            return input.to_owned();
        }
        println!("{}", "invalid input".red().bold());
    }
}

pub fn select_device_by_prompt(host: &cpal::Host) -> cpal::Device{
    let _print_gag = Gag::stderr();
 
    println!("Select input or output device");
    let prompt = format!("Enter {} for input device, {} for output device", "1".green().bold(), "2".green().bold());
    let input_device = prompt_user_input(&["1".to_string(), "2".to_string()], prompt);
    println!();
    
    let devices: Vec<cpal::Device> = if input_device == "1" {
        println!("Select an input device");
        host.input_devices()
            .unwrap()
            .collect::<Vec<cpal::Device>>()
    } else {
        println!("Select an output device");
        host.output_devices()
            .unwrap()
            .collect::<Vec<cpal::Device>>()
    };
    
    let device_indices = (0..devices.len()).map(|index| index.to_string()).collect::<Vec<String>>();    
    for (index, device) in devices.iter().enumerate() {
        match device.name() {
            Ok(name) => {
                println!("{} {}", index.to_string().green().bold(), name);
            },
            Err(_) => {
                println!("{} Unknown Device", index.to_string().green().bold());
            }
        }
    }

    println!();
    let prompt = format!("Enter a number between {} and {}", "0".green().bold(), (devices.len() - 1).to_string().green().bold());
    let device_index = prompt_user_input(&device_indices, prompt);
    println!();
    
    devices[device_index.parse::<usize>().unwrap()].clone()
}