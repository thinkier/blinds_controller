#![no_std]

#[cfg(test)]
mod tests;

/// Soft-float compatible naive thermistor calculator
pub struct Thermistor {
    pub ref_resistance: f32,
    pub ref_temp_kelvin: f32,
    pub beta: f32,
}

impl Thermistor {
    pub const fn new_celsius(ref_resistance: f32, ref_temp_celsius: f32, beta: f32) -> Thermistor {
        Thermistor {
            ref_resistance,
            ref_temp_kelvin: ref_temp_celsius + 273.15,
            beta,
        }
    }

    pub fn get_temp_celsius(&self, resistance: f32) -> f32 {
        self.get_temp_kelvin(resistance) - 273.15
    }
}

impl Thermistor {
    /// $$
    /// \beta = \frac{ln(\frac{R_{ref}}{R_{measured}})}{T_{ref}^{-1} - T_{measured}^{-1}}
    /// $$
    ///
    /// $$
    /// \therefore
    /// T =\frac{1}{\frac{1}{T_{ref}} + \frac{1}{\beta}ln(\frac{R_{ref}}{R})}
    /// $$
    pub fn get_temp_kelvin(&self, resistance: f32) -> f32 {
        // Written in prefix notation like the LaTeX
        let log_res = ln(self.ref_resistance / resistance);
        inv(inv(self.ref_temp_kelvin) + (inv(self.beta) * log_res))
    }
}

fn ln(item: f32) -> f32 {
    libm::logf(item)
}

fn inv(item: f32) -> f32 {
    item.recip()
}
