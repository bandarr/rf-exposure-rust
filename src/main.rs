struct FrequencyValues {
    freq: f32,
    swr: f32,
    gaindbi: f32,
}

struct CableValues {
    k1: f32,
    k2: f32,
}

fn main() {
    let cable_values: CableValues = CableValues {
        k1: 0.122290,
        k2: 0.000260,
    };

    let all_frequency_values: [FrequencyValues; 6] = [
        FrequencyValues {
            freq: 7.3,
            swr: 2.25,
            gaindbi: 1.5,
        },
        FrequencyValues {
            freq: 14.35,
            swr: 1.35,
            gaindbi: 1.5,
        },
        FrequencyValues {
            freq: 18.1,
            swr: 3.7,
            gaindbi: 1.5,
        },
        FrequencyValues {
            freq: 21.45,
            swr: 4.45,
            gaindbi: 1.5,
        },
        FrequencyValues {
            freq: 24.99,
            swr: 4.1,
            gaindbi: 1.5,
        },
        FrequencyValues {
            freq: 29.7,
            swr: 2.18,
            gaindbi: 4.5,
        },
    ];

    const XMTR_POWER: i32 = 1000;
    const FEEDLINE_LENGTH: i32 = 73;
    const DUTY_CYCLE: f32 = 0.5;
    const PER_30: f32 = 0.5;

    all_frequency_values.iter().for_each(|f| {
        let yarg = calculate_uncontrolled_safe_distance(
            f,
            &cable_values,
            XMTR_POWER,
            FEEDLINE_LENGTH,
            DUTY_CYCLE,
            PER_30,
        );
        println!("{:.2}", yarg);
    });
}

fn calculate_uncontrolled_safe_distance(
    freq_values: &FrequencyValues,
    cable_values: &CableValues,
    transmitter_power: i32,
    feedline_length: i32,
    duty_cycle: f32,
    uncontrolled_percentage_30_minutes: f32,
) -> f32 {
    let gamma = calculate_reflection_coefficient(freq_values);

    let feedline_loss_per_100ft_at_frequency =
        calculate_feedline_loss_per_100ft_at_frequency(freq_values, cable_values);

    let feedline_loss_for_matched_load_at_frequency =
        calculate_feedline_loss_for_matched_load_at_frequency(
            feedline_length,
            feedline_loss_per_100ft_at_frequency,
        );

    let feedline_loss_for_matched_load_at_frequency_percentage =
        calculate_feedline_loss_for_matched_load_at_frequency_percentage(
            feedline_loss_for_matched_load_at_frequency,
        );

    let gamma_squared = gamma.abs().powf(2.0);

    let feedline_loss_for_swr = calculate_feedline_loss_for_swr(
        feedline_loss_for_matched_load_at_frequency_percentage,
        gamma_squared,
    );

    let feedline_loss_for_swr_percentage =
        calculate_feedline_loss_for_swr_percentage(feedline_loss_for_swr);

    let power_loss_at_swr = feedline_loss_for_swr_percentage * transmitter_power as f32;

    let peak_envelope_power_at_antenna = transmitter_power as f32 - power_loss_at_swr;

    let uncontrolled_average_pep =
        peak_envelope_power_at_antenna * duty_cycle * uncontrolled_percentage_30_minutes;

    let mpe_s = 180.0 / (freq_values.freq.powf(2.0));

    let gain_decimal = 10.0_f32.powf(freq_values.gaindbi / 10.0);

    return ((0.219 * uncontrolled_average_pep * gain_decimal) / mpe_s).sqrt();
}

fn calculate_reflection_coefficient(freq_values: &FrequencyValues) -> f32 {
    return ((freq_values.swr - 1.0) / (freq_values.swr + 1.0)).abs();
}

fn calculate_feedline_loss_for_matched_load_at_frequency(
    feedline_length: i32,
    feedline_loss_per_100ft_at_frequency: f32,
) -> f32 {
    return (feedline_length as f32 / 100.0) * feedline_loss_per_100ft_at_frequency;
}

fn calculate_feedline_loss_for_matched_load_at_frequency_percentage(
    feedline_loss_for_matched_load: f32,
) -> f32 {
    return 10.0_f32.powf(-feedline_loss_for_matched_load / 10.0);
}

fn calculate_feedline_loss_per_100ft_at_frequency(
    freq_values: &FrequencyValues,
    cable_values: &CableValues,
) -> f32 {
    return cable_values.k1 * (freq_values.freq + cable_values.k2 * freq_values.freq).sqrt();
}

fn calculate_feedline_loss_for_swr(
    feedline_loss_for_matched_load_percentage: f32,
    gamma_squared: f32,
) -> f32 {
    return -10.0
        * (feedline_loss_for_matched_load_percentage
            * ((1.0 - gamma_squared)
                / (1.0 - feedline_loss_for_matched_load_percentage.powf(2.0) * gamma_squared)))
            .log10();
}

fn calculate_feedline_loss_for_swr_percentage(feedline_loss_for_swr: f32) -> f32 {
    return (100.0 - 100.0 / (10.0_f32.powf(feedline_loss_for_swr / 10.0))) / 100.0;
}
