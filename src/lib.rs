use chashmap::CHashMap;
use std::{fmt, time::Duration, io::{self, prelude::*}, fs::File};

pub struct Stats {
    pub pings_to_peers: CHashMap<String, Vec<Duration>>,
    pub transmissions_rates: CHashMap<String, Vec<Duration>>,
    window_size: usize,
    peer_id: String,
}

impl Stats {
    pub fn new(window_size: usize, peer_id: String) -> Self {
        Self {
            pings_to_peers: CHashMap::new(),
            transmissions_rates: CHashMap::new(),
            window_size,
            peer_id,
        }
    }

    pub fn save_to_file(&self, filename: &str) -> io::Result<()>{
        let mut file = File::create(filename)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }
}

fn durations_mean(durations: &Vec<Duration>) -> Option<Duration> {
    if durations.is_empty() {
        None
    } else {
        Some(
            durations
                .iter()
                .fold(Duration::from_secs(0), |acc, x| acc + *x)
                / durations.len() as u32,
        )
    }
}

#[test]
fn correct_durations_mean() {
    let durations = vec![
        Duration::from_secs(1),
        Duration::from_secs(3),
        Duration::from_secs(5),
    ];
    assert_eq!(durations_mean(&durations).unwrap(), Duration::from_secs(3));
}

fn durations_std_dev(durations: &Vec<Duration>) -> Option<Duration> {
    let mean = durations_mean(durations)?.as_secs_f64();
    Some(Duration::from_secs_f64(
        (durations
            .iter()
            .fold(0f64, |acc, x| acc + (x.as_secs_f64() - mean).powi(2))
            / (durations.len() as f64))
            .sqrt(),
    ))
}

#[test]
fn correct_durations_std_dev() {
    let durations = vec![
        Duration::from_secs(1),
        Duration::from_secs(3),
        Duration::from_secs(5),
    ];
    let epsilon = 0.01;
    let std_dev = durations_std_dev(&durations).unwrap().as_secs_f64();
    assert!((std_dev - 1.63).abs() < epsilon);
}

/// Durations mean error with confidence interval of 95%
/// For correct estimation `durations.len()` should be at least `30`.
fn durations_error_with_ci(durations: &Vec<Duration>) -> Option<Duration> {
    // Z-value for 95 percent confidence interval
    let z = 1.96;
    let std_dev = durations_std_dev(durations)?;
    Some(Duration::from_secs_f64(
        z * std_dev.as_secs_f64() / (durations.len() as f64).sqrt(),
    ))
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ping_by_peer: String = self
            .pings_to_peers
            .clone()
            .into_iter()
            .map(|(peer, durations)| {
                match (
                    durations_mean(&durations),
                    durations_error_with_ci(&durations),
                ) {
                    (Some(duration), Some(error)) => {
                        format!("{:?} {:?}±{:?}\n", peer, duration, error)
                    }
                    _ => format!("No ping data for peer {:?}", peer),
                }
            })
            .collect();

        let transmission_rate_by_peer: String = self
            .transmissions_rates
            .clone()
            .into_iter()
            .map(|(peer, durations)| {
                match (
                    durations_mean(&durations),
                    durations_error_with_ci(&durations),
                ) {
                    (Some(duration), Some(error)) => {
                        format!("{:?} {:?}±{:?} per byte\n", peer, duration, error)
                    }
                    _ => format!("No transmission data for peer {:?}", peer),
                }
            })
            .collect();
        write!(
            f,
            "{:?}\nPing mean for each peer:\n{}Transmission rate mean by peer:\n{}",
            self.peer_id, ping_by_peer, transmission_rate_by_peer
        )
    }
}

pub trait PushLossy<T> {
    fn push_lossy(&mut self, element: T, window_size: usize);
}

impl<T> PushLossy<T> for Vec<T> {
    fn push_lossy(&mut self, element: T, window_size: usize) {
        if self.len() >= window_size {
            self.remove(0);
        }
        self.push(element);
    }
}
