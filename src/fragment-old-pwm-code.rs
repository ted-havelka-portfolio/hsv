/*
    // - STEP - Set up PWM for blue LED:
    let pwm_pin = board.edge.e08.into_push_pull_output(Level::Low).degrade();

    // Configuring output pin
    let pwm_led_blu = pwm::Pwm::new(board.PWM0);
    pwm_led_blu.set_output_pin(pwm::Channel::C0, pwm_pin);
    pwm_led_blu.set_prescaler(pwm::Prescaler::Div32);
    pwm_led_blu.set_max_duty(LED_DUTY_MAX);
    pwm_led_blu.loop_inf();

    // Print the period in order to check the configuration
    rprintln!("green led period is {} us", pwm_led_blu.period().0);

    // Define duty cycle
    let mut duty = 5_000_u16;
    let mut duty_blu;
    pwm_led_blu.set_duty_off(pwm::Channel::C0, duty);

    // - STEP - Set up PWM for green LED:
    let pwm_pin = board.edge.e09.into_push_pull_output(Level::Low).degrade();

    // Configuring output pin
    let pwm_led_grn = pwm::Pwm::new(board.PWM1);
    pwm_led_grn.set_output_pin(pwm::Channel::C1, pwm_pin);
    pwm_led_grn.set_prescaler(pwm::Prescaler::Div32);
    pwm_led_grn.set_max_duty(LED_DUTY_MAX);
    pwm_led_grn.loop_inf();

    // - STEP - Set up PWM for red LED:
    let pwm_pin = board.edge.e12.into_push_pull_output(Level::Low).degrade();

    // Configuring output pin
    let pwm_led_red = pwm::Pwm::new(board.PWM2);
    pwm_led_red.set_output_pin(pwm::Channel::C2, pwm_pin);
    pwm_led_red.set_prescaler(pwm::Prescaler::Div32);
    pwm_led_red.set_max_duty(LED_DUTY_MAX);
    pwm_led_red.loop_inf();
*/


/*
        state = match state {
            State::LedOff => {
                // edge08.set_high().unwrap();
                duty_blu = 440_u16;
                // edge09.set_low().unwrap();
                rprintln!("high");
                State::LedOn
            }
            State::LedOn => {
                // edge08.set_low().unwrap();
                duty_blu = 5_000_u16;
                // edge09.set_high().unwrap();
                rprintln!("low");
                State::LedOff
            }
        };

        for duty_cycle in led_intensities_grn.iter() {
            duty = (LED_DUTY_MAX as f32 * (*duty_cycle as f32 / 100.0)) as u16;
            pwm_led_grn.set_duty_on(pwm::Channel::C1, duty);
            rprintln!("LED green duty cycle {}", duty);
            timer.delay_ms(100);
        }

        // pwm_led_blu.set_duty_off(pwm::Channel::C0, duty);
        pwm_led_blu.set_duty_on(pwm::Channel::C0, duty_blu);
        timer.delay_ms(500);

        let duty_red = duty_blu;
        pwm_led_red.set_duty_on(pwm::Channel::C2, duty_red);
        timer.delay_ms(500);
        pwm_led_red.set_duty_on(pwm::Channel::C2, 10);
*/

