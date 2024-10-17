pub fn growth(neighbours: f64, bell_m: f64, bell_s: f64) -> f64 {
    bell(neighbours, bell_m, bell_s) * 2.0 - 1.0
}

pub fn bell(x: f64, m: f64, s: f64) -> f64 {
    f64::exp(-((x - m) / s).powi(2) / 2.0)
}
