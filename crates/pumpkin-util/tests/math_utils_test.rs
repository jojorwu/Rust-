use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::euler_angle::EulerAngle;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

#[test]
fn test_vector3_operations() {
    let v1 = Vector3::new(1.0, 2.0, 3.0);
    let v2 = Vector3::new(4.0, 5.0, 6.0);

    let sum = v1 + v2;
    assert_eq!(sum.x, 5.0);
    assert_eq!(sum.y, 7.0);
    assert_eq!(sum.z, 9.0);

    let sub = v2 - v1;
    assert_eq!(sub.x, 3.0);
    assert_eq!(sub.y, 3.0);
    assert_eq!(sub.z, 3.0);

    let scaled = v1 * 2.0;
    assert_eq!(scaled.x, 2.0);
    assert_eq!(scaled.y, 4.0);
    assert_eq!(scaled.z, 6.0);
}

#[test]
fn test_block_pos_conversion_and_math() {
    let pos = BlockPos::new(10, -5, 20);
    assert_eq!(pos.0.x, 10);
    assert_eq!(pos.0.y, -5);
    assert_eq!(pos.0.z, 20);

    let offset_pos = pos.add(1, 2, 3);
    assert_eq!(offset_pos.0.x, 11);
    assert_eq!(offset_pos.0.y, -3);
    assert_eq!(offset_pos.0.z, 23);
}

#[test]
fn test_bounding_box_intersections() {
    let bb1 = BoundingBox::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(2.0, 2.0, 2.0),
    );
    let bb2 = BoundingBox::new(
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(3.0, 3.0, 3.0),
    );
    let bb3 = BoundingBox::new(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(6.0, 6.0, 6.0),
    );

    assert!(bb1.intersects(&bb2));
    assert!(bb2.intersects(&bb1));
    assert!(!bb1.intersects(&bb3));
}

#[test]
fn test_euler_angle() {
    let angle = EulerAngle::new(45.0, 90.0, 180.0);
    assert_eq!(angle.pitch, 45.0);
    assert_eq!(angle.yaw, 90.0);
    assert_eq!(angle.roll, 180.0);
}
