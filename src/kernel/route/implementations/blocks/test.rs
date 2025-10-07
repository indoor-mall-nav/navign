#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_bounded_block() {
        let block = BoundedBlock {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            is_bounded: true,
        };
        assert!(block.is_point_inside(5.0, 5.0));
        assert!(!block.is_point_inside(15.0, 5.0));
    }

    #[test]
    fn test_ray_cast() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice());
        assert!(poly.is_point_inside(0.5, 0.5));
        assert!(poly.is_point_inside(1.5, 0.5));
        assert!(!poly.is_point_inside(4.0, 0.5));
    }

    #[test]
    fn test_ray_cast_unbound() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice()).into_unbound();
        assert!(!poly.is_point_inside(0.5, 0.5));
        assert!(!poly.is_point_inside(1.5, 0.5));
        assert!(poly.is_point_inside(4.0, 0.5));
    }

    #[test]
    fn test_get_sorted_coords() {
        let polygon = &[(2.0, 3.0), (0.0, 1.0), (1.0, 2.0)];
        let poly = Polygon::from(polygon.as_slice());
        let (xs, ys) = poly.get_sorted_coords();
        assert_eq!(xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(ys, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_bounded_block_array() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 0.0,
                y1: 1.0,
                x2: 1.0,
                y2: 2.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 1.0,
                x2: 2.0,
                y2: 2.0,
                is_bounded: false,
            },
        ];
        let array = BoundedBlockArray {
            blocks: &blocks,
            memory_width: 2,
            memory_height: 2,
            width: 2.0,
            height: 2.0,
        };

        assert_eq!(array.memory_width(), 2);
        assert_eq!(array.memory_height(), 2);

        let block = array.fit(0.5, 0.5).unwrap();
        assert_eq!(block.x1, 0.0);
        assert_eq!(block.y1, 0.0);

        let up_block = array.up(block);
        assert!(up_block.is_none());

        let down_block = array.down(block).unwrap();
        assert_eq!(down_block.x1, 0.0);
        assert_eq!(down_block.y1, 1.0);

        let right_block = array.right(block).unwrap();
        assert_eq!(right_block.x1, 1.0);
        assert_eq!(right_block.y1, 0.0);

        let left_block = array.left(block);
        assert!(left_block.is_none());
    }

    #[test]
    fn test_bounded_block_array_coords() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 0.0,
                y1: 1.0,
                x2: 1.0,
                y2: 2.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 1.0,
                x2: 2.0,
                y2: 2.0,
                is_bounded: false,
            },
        ];
        let array = BoundedBlockArray {
            blocks: &blocks,
            memory_width: 2,
            memory_height: 2,
            width: 2.0,
            height: 2.0,
        };

        let xs = array.coords_x();
        assert_eq!(xs, vec![0.0, 1.0, 2.0]);

        let ys = array.coords_y();
        assert_eq!(ys, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_polygon_to_bounded_block_array() {
        let polygon = &[(0.0, 0.0), (0.0, 2.0), (2.0, 2.0), (2.0, 0.0), (0.0, 0.0)];
        let poly = Polygon::from(polygon.as_slice());
        let array = poly.as_bounded_block_array();

        assert_eq!(array.memory_width, 1);
        assert_eq!(array.memory_height, 1);
        assert_eq!(array.width, 2.0);
        assert_eq!(array.height, 2.0);

        for block in array.blocks.iter() {
            assert!(block.is_bounded);
        }
    }

    #[test]
    fn test_advanced_polygon_to_bounded_block_array() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice());
        let array = poly.as_bounded_block_array();

        assert_eq!(array.memory_width, 2);
        assert_eq!(array.memory_height, 2);

        assert_eq!(array.width, 2.0);
        assert_eq!(array.height, 3.0);

        let mut bounded_count = 0;
        let mut unbounded_count = 0;

        for block in array.blocks.iter() {
            if block.is_bounded {
                bounded_count += 1;
            } else {
                unbounded_count += 1;
            }
        }

        assert_eq!(bounded_count, 3);
        assert_eq!(unbounded_count, 1);
    }
}
