(define (problem magnets-1)
  (:domain magnets)

  (:objects
    m1 m2 - magnet
    l0 l1 l2 - coord
  )

  (:init
    (at m1 l0 l0)
    (at m2 l1 l1)

    (different m1 m2)
    (different m2 m1)

    (forbidden l2 l2)

    ;; 4-ary grid adjacency on a 3x3 grid (Manhattan distance = 1).
    ;; Horizontal edges (same x, y differs by 1)
    (adj l0 l0 l0 l1) (adj l0 l1 l0 l0)
    (adj l0 l1 l0 l2) (adj l0 l2 l0 l1)
    (adj l1 l0 l1 l1) (adj l1 l1 l1 l0)
    (adj l1 l1 l1 l2) (adj l1 l2 l1 l1)
    (adj l2 l0 l2 l1) (adj l2 l1 l2 l0)
    (adj l2 l1 l2 l2) (adj l2 l2 l2 l1)

    ;; Vertical edges (same y, x differs by 1)
    (adj l0 l0 l1 l0) (adj l1 l0 l0 l0)
    (adj l1 l0 l2 l0) (adj l2 l0 l1 l0)
    (adj l0 l1 l1 l1) (adj l1 l1 l0 l1)
    (adj l1 l1 l2 l1) (adj l2 l1 l1 l1)
    (adj l0 l2 l1 l2) (adj l1 l2 l0 l2)
    (adj l1 l2 l2 l2) (adj l2 l2 l1 l2)
  )

  (:goal (and
    (at m1 l0 l2)
    (at m2 l0 l2)
  ))
)
