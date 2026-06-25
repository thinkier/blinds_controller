#![no_std]

#[cfg(test)]
mod tests;

/// Pre-filled for the ERT-J??G line with $\beta_{25/85}$
/// https://industrial.panasonic.com/cdbs/www-data/pdf/AUA0000/AUA0000C8.pdf
pub const ERT_J1VGXXA: NtcThermistor = NtcThermistor::new_celsius(1e4, 25., 3435.);

/// Pre-filled for the EPCOS 100K line with $\beta_{25/85}$
/// https://www.mouser.com/catalog/specsheets/glass_enc_sensors__b57560__g560__g1560.pdf
///
/// Most commonly used for cheap 3D printers
// I added this for debugging because I have a handful of spares tinkering with an Ender 3 clone
pub const EPCOS_100K: NtcThermistor = NtcThermistor::new_celsius(1e5, 25., 4072.);

/// Soft-float compatible naive thermistor calculator
pub struct NtcThermistor {
    pub ref_resistance: f32,
    pub ref_temp_kelvin: f32,
    pub beta: f32,
}

impl NtcThermistor {
    pub const fn new_celsius(ref_resistance: f32, ref_temp_celsius: f32, beta: f32) -> NtcThermistor {
        NtcThermistor {
            ref_resistance,
            ref_temp_kelvin: ref_temp_celsius + 273.15,
            beta,
        }
    }

    pub fn get_temp_celsius(&self, resistance: f32) -> f32 {
        self.get_temp_kelvin(resistance) - 273.15
    }
}

impl NtcThermistor {
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
