#![no_std]

#[allow(unused_imports)]
use micromath::F32Ext;

pub fn physics_acceleration(a: f32, v: f32, d: f32) -> (f32, f32) {
    let v2 = v * v;
    let t = -v + ((2.0 * a * d) + v2).sqrt();
    let t_s = t / a;
    return (t_s, (a*t_s) + v);
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn test_physics_delta_initial() {
        let (t, v) = physics_acceleration(0.01, 0.0, 1.0);
        assert_relative_eq!(t, 14.1421355, epsilon = 0.0001);
        assert_relative_eq!(v, 0.1414213562, epsilon = 0.0001);
    }

    #[test]
    fn test_physics_delta_i1() {
        let (t, v) = physics_acceleration(0.01, 0.1414213562, 1.0);
        assert_relative_eq!(t, 5.857864376, epsilon = 0.0001);
        assert_relative_eq!(v, 0.2, epsilon = 0.0001);
    }

    #[test]
    fn test_physics_10_iterations() {
        let mut t;
        let mut v = 0.0;
        let expect_t = [
            14.14213562,
            5.857864376,
            4.494897428,
            3.78937382,
            3.338505354,
            3.01823955, 
            2.775557716,
            2.583426132,
            2.426406871,
            2.294952679,
            2.182798048,
            2.085637257,
            2.00040028,
            1.924831085,
            1.857229529,
        ];
        let expect_v = [
            0.1414213562,
            0.2,
            0.2449489743,
            0.2828427125,
            0.316227766,
            0.3464101615,
            0.3741657387,
            0.4,
            0.4242640687,
            0.4472135955,
            0.469041576,
            0.489897949,
            0.509901951,
            0.529150262,
            0.547722558
        ];
        for i in 0..15 {
            (t, v) = physics_acceleration(0.01, v, 1.0);
            assert_relative_eq!(t, expect_t[i], epsilon = 0.0001);
            assert_relative_eq!(v, expect_v[i], epsilon = 0.0001);
        }
    }
}
