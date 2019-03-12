extern crate csv;
extern crate serde_derive;

//use std::sync::{Arc, Mutex}; //TODO Concurrency
//use std::thread;

mod structs;

use std::collections::HashMap;
use std::time::Instant;
use structs::*;

fn make_partitions<T: Data<T> + Clone + Copy>(data: &Vec<T>, folds: usize) -> Vec<Vec<T>> {
    let mut categories_count = HashMap::new();
    let mut partitions: Vec<Vec<T>> = Vec::new();

    for _ in 0..folds {
        partitions.push(Vec::new());
    }

    for example in data {
        let counter = categories_count.entry(example.get_class()).or_insert(0);
        partitions[*counter].push(example.clone());

        *counter = (*counter + 1) % folds;
    }

    return partitions;
}

pub fn _1_nn<T: Data<T> + Clone + Copy>(
    data: &Vec<Vec<T>>,
    folds: usize,
) -> Result<(), Box<std::error::Error>> {
    let begin = Instant::now();

    // NOTE For each partition
    for i in 0..folds {
        // NOTE Test over partition i
        let mut _correct = 0;
        let mut _attempts = 0;

        let mut knowledge: Vec<T> = Vec::new();

        // Learning from all other partitions.
        for j in 0..folds {
            if j != i {
                knowledge.extend(&data[j]);
            }
        }

        // NOTE Test
        for result in data[i].iter() {
            _attempts += 1;

            let mut nearest_example: T = T::new();
            let mut min_distance: f32 = std::f32::MAX;

            for known in knowledge.iter() {
                let distance = result.euclidean_distance(known);

                if distance < min_distance {
                    min_distance = distance;
                    nearest_example = known.clone();
                }
            }

            if nearest_example.get_class() == result.get_class() {
                _correct += 1;
            }
        }

        println!(
            "Total aciertos en test {}: {}/{} = {}",
            i,
            _correct,
            _attempts,
            _correct as f32 / _attempts as f32
        );
    }
    println!(
        "Tiempo transcurrido: {} milisegundos.",
        begin.elapsed().as_millis()
    );

    Ok(())
}

pub fn relief<T: Data<T> + Clone + Copy>(
    data: &Vec<Vec<T>>,
    folds: usize,
    atributes: usize,
    discard_low_weights: bool,
) -> Result<(), Box<std::error::Error>> {
    let begin = Instant::now();
    let mut total_correct = 0;

    let mut _weights: Vec<f32> = Vec::new();
    for _ in 0..atributes {
        _weights.push(0.0);
    }

    // NOTE For each partition
    for i in 0..folds {
        let mut _correct = 0;
        let mut _attempts = 0;

        // NOTE Test over partition i
        let mut knowledge: Vec<T> = Vec::new();

        // Learning from all other partitions.
        for j in 0..folds {
            if j != i {
                knowledge.extend(data[j].iter().cloned());
            }
        }

        // NOTE Greedy
        for known in knowledge.iter() {
            let mut enemy: T = T::new();
            let mut ally: T = T::new();
            let mut enemy_distance = std::f32::MAX;
            let mut friend_distance = std::f32::MAX;

            for candidate in knowledge.iter() {
                //NOTE Skip if cantidate == known
                if candidate.get_id() != known.get_id() {
                    // NOTE Pre-calculate distance
                    let dist = known.euclidean_distance(candidate);
                    // NOTE Ally
                    if known.get_class() == candidate.get_class() {
                        if dist < friend_distance {
                            ally = candidate.clone();
                            friend_distance = dist;
                        }
                    }
                    // NOTE Enemy
                    else {
                        if dist < enemy_distance {
                            enemy = candidate.clone();
                            enemy_distance = dist;
                        }
                    }
                }
            }
            // NOTE Re-calculate weights
            let mut highest_weight: f32 = _weights[0];
            for attr in 0..atributes {
                _weights[attr] += f32::abs(known.get_attr(attr) - enemy.get_attr(attr))
                    - f32::abs(known.get_attr(attr) - ally.get_attr(attr));

                // NOTE Find maximal element for future normalizing.
                if _weights[attr] > highest_weight {
                    highest_weight = _weights[attr];
                }
                // NOTE Truncate negative values as 0.
                if _weights[attr] < 0.0 {
                    _weights[attr] = 0.0;
                }
            }

            // NOTE Normalize weights
            for attr in 0..atributes {
                _weights[attr] = _weights[attr] / highest_weight;
            }
        } // NOTE END Greedy

        // NOTE Test
        for result in data[i].iter() {
            _attempts += 1;

            let mut nearest_example: T = T::new();
            let mut min_distance: f32 = std::f32::MAX;

            for known in knowledge.iter() {
                //NOTE Distance with weights
                let mut distance = 0.0;
                for index in 0..atributes {
                    // NOTE weights below 0.2 aren't considered.
                    if discard_low_weights || _weights[index] >= 0.2 {
                        distance += _weights[index]
                            * (result.get_attr(index) - known.get_attr(index))
                            * (result.get_attr(index) - known.get_attr(index))
                    }
                }

                distance = distance.sqrt();

                if distance < min_distance {
                    min_distance = distance;
                    nearest_example = known.clone();
                }
            }

            if nearest_example.get_class() == result.get_class() {
                _correct += 1;
            }
        }

        total_correct += _correct;

        println!(
            "Total aciertos en test {}: {}/{} = {}",
            i,
            _correct,
            _attempts,
            _correct as f32 / _attempts as f32
        );
    }
    println!(
        "Tiempo transcurrido: {} milisegundos.",
        begin.elapsed().as_millis()
    );

    //println!("Vector de pesos: {:?}", _weights);
    let mut reduction: f32 = 0.0;

    for w in _weights {
        if w < 0.2 {
            reduction += 1.0;
        }
    }
    reduction = 100.0 * reduction / 40.0;

    let f = 0.5 * reduction + 0.5 * (100.0 * total_correct as f32 / 550.0);

    println!(
        "Tasa Reducción: {} \nFunción de evaluación: {}",
        reduction, f
    );

    Ok(())
}

pub fn _1_nn_texture() -> Result<(), Box<std::error::Error>> {
    let mut csv_reader = csv::Reader::from_path("data/csv_result-texture.csv")?;
    let mut data: Vec<Texture> = Vec::new();
    let _atributes: usize = 40;

    // NOTE CSV -> Data.
    for result in csv_reader.records() {
        let mut aux_record = Texture::new();
        let record = result?;
        let mut counter = 0;

        for field in record.iter() {
            // NOTE CSV structure: id , ... 40 data ... , class
            if counter == 0 {
                aux_record._id = field.parse::<i32>().unwrap();
            } else if counter != 41 {
                aux_record._attrs[counter - 1] = field.parse::<f32>().unwrap();
            } else {
                aux_record._class = field.parse::<i32>().unwrap();
            }

            counter += 1;
        }

        data.push(aux_record);
    }

    let data: Vec<Vec<Texture>> = make_partitions(&data, 5);
    println!("Resultados 1-NN:");
    return _1_nn(&data, 5);
}

pub fn relief_texture() -> Result<(), Box<std::error::Error>> {
    let mut csv_reader = csv::Reader::from_path("data/csv_result-texture.csv")?;
    let mut data: Vec<Texture> = Vec::new();
    let _atributes: usize = 40;

    // NOTE CSV -> Data.
    for result in csv_reader.records() {
        let mut aux_record = Texture::new();
        let record = result?;
        let mut counter = 0;

        for field in record.iter() {
            // CSV structure: id , ... 40 data ... , class
            if counter == 0 {
                aux_record._id = field.parse::<i32>().unwrap();
            } else if counter != 41 {
                aux_record._attrs[counter - 1] = field.parse::<f32>().unwrap();
            } else {
                aux_record._class = field.parse::<i32>().unwrap();
            }

            counter += 1;
        }

        data.push(aux_record);
    }

    let data: Vec<Vec<Texture>> = make_partitions(&data, 5);

    println!("Resultados Relief descartando pesos < 0.2:");
    relief(&data, 5, 40, true)?;

    println!("Resultados Relief sin descartar pesos < 0.2:");
    return relief(&data, 5, 40, false);
}

fn main() {
    if let Err(err) = _1_nn_texture() {
        println!("error running 1-nn: {}", err);
        std::process::exit(1);
    }

    if let Err(err) = relief_texture() {
        println!("error running relief: {}", err);
        std::process::exit(1);
    }
}
