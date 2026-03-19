# Car Mechanics

## Driving State Machine

* *Coast*: The car is coasting, neither accelerating nor braking. Engine break torque is applied. Target motor spin is 0. The car transitions into Forward or Braking as long as there is no considerable backward velocity (v_long > -v_backward), and into Reverse or Braking if there is a considerable backward velocity (v_long < -v_backward).
* *Forward*: The car is accelerating forward, as long as "throttle" is pressed. Positive drive torque is applied to driven wheels. Target motor spin is based on current forward gear.
* *Forward(Braking)*: Same as *Forward* but with applied brakes. Positive drive torque is applied to driven wheels. Target motor spin is based on current forward gear. Brake torque is applied to all non-driven wheels. Target motor spin is 0.
* *Reverse*: The car is accelerating backward, as long as "brake" is pressed. Negative drive torque is applied to driven wheels. Target motor spin is based on reverse gear.
* *Reverse(Braking)*: Same as *Reverse* but with applied brakes. Negative drive torque is applied to driven wheels. Target motor spin is based on reverse gear. Brake torque is applied to all non-driven wheels. Target motor spin is 0.
* *Braking*: The car is applying brakes, transitioning into Stopped state when velocity reaches near zero (v_long < v_epsilon). Brake torque is applied to all wheels. Target motor spin is 0.
* *Stopped*: The car is temporarily stationary, transitioning into the opposite direction after a delay. Brake torque is applied to all wheels. Target motor spin is 0.

Variables used:
* v_long = dot(v, forward_dir)
* v_epsilon = Speed at which the car is considered to be stationary (near zero velocity, i.e. 0.1 m/s).
* v_backward = Speed at which the car is considered to be moving backwards. (i.e. -0.5 m/s).
* delay = Minimum time the car must be stationary before switching to the opposite direction is allowed.

### State Transitions

    State            | condition                | throttle | brake | Next State
    Coast            | v_long >= v_backward     | -        | -     | Coast
    Coast            | v_long >= v_backward     | x        | -     | Forward
    Coast            | v_long >= v_backward     | -        | x     | Braking(Forward)
    Coast            | v_long >= v_backward     | x        | x     | Forward(Braking)
    Coast            | v_long < v_backward      | -        | -     | Coast
    Coast            | v_long < v_backward      | x        | -     | Braking(Reverse)
    Coast            | v_long < v_backward      | -        | x     | Reverse
    Coast            | v_long < v_backward      | x        | x     | Reverse(Braking)
    Forward          | -                        | -        | -     | Coast
    Forward          | -                        | x        | -     | Forward
    Forward          | -                        | -        | x     | Coast
    Forward          | -                        | x        | x     | Forward(Braking)
    Forward(Braking) | -                        | -        | -     | Coast
    Forward(Braking) | -                        | x        | -     | Forward
    Forward(Braking) | -                        | -        | x     | Braking(Forward)
    Forward(Braking) | -                        | x        | x     | Forward(Braking)
    Reverse          | -                        | -        | -     | Coast
    Reverse          | -                        | x        | -     | Coast
    Reverse          | -                        | -        | x     | Reverse
    Reverse          | -                        | x        | x     | Reverse(Braking)
    Reverse(Braking) | -                        | -        | -     | Coast
    Reverse(Braking) | -                        | x        | -     | Braking(Reverse)
    Reverse(Braking) | -                        | -        | x     | Reverse
    Reverse(Braking) | -                        | x        | x     | Reverse(Braking)
    Braking(Forward) | abs(v_long) < v_epsilon  | -        | x     | Stopped
    Braking(Forward) | -                        | -        | -     | Coast
    Braking(Forward) | -                        | x        | -     | Forward
    Braking(Forward) | -                        | -        | x     | Braking(Forward)
    Braking(Forward) | -                        | x        | x     | Forward(Braking)
    Braking(Reverse) | abs(v_long) < v_epsilon  | x        | -     | Stopped
    Braking(Reverse) | -                        | -        | -     | Coast
    Braking(Reverse) | -                        | x        | -     | Braking(Reverse)
    Braking(Reverse) | -                        | -        | x     | Reverse
    Braking(Reverse) | -                        | x        | x     | Reverse(Braking)
    Stopped          | state_time >= delay      | x        | -     | Forward
    Stopped          | state_time >= delay      | -        | x     | Reverse
    Stopped          | -                        | -        | -     | Coast
    Stopped          | -                        | x        | -     | Stopped
    Stopped          | -                        | -        | x     | Stopped
    Stopped          | -                        | x        | x     | Coast
