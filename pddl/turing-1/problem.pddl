(define (problem turing-machine-1)
  (:domain turing-machine)

  (:objects
    p0 p1 p2 p3 p4 p5 p6 p7 - position
    s0 s1 s2 s3 - state
    zero one blank - symbol
    left right - direction
  )

  (:init
    (head p1)
    (current-state s0)

    (tape p0 blank)
    (tape p1 zero)
    (tape p2 one)
    (tape p3 zero)
    (tape p4 one)
    (tape p5 one)
    (tape p6 zero)
    (tape p7 blank)

    ;; Tape adjacency
    (next p0 p1 right) (next p1 p2 right) (next p2 p3 right)
    (next p3 p4 right) (next p4 p5 right) (next p5 p6 right)
    (next p6 p7 right)
    (next p1 p0 left) (next p2 p1 left) (next p3 p2 left)
    (next p4 p3 left) (next p5 p4 left) (next p6 p5 left)
    (next p7 p6 left)

    (accepting s3)
    (is-blank blank)

    ;; Transition function: delta(state, read, next-state, write, direction)
    ;; s0
    (delta s0 zero s0 zero right)
    (delta s0 one  s0 one  right)
    (delta s0 one  s1 zero right)
    (delta s0 zero s2 one  right)
    ;; s1
    (delta s1 zero s1 zero right)
    (delta s1 one  s1 one  right)
    (delta s1 zero s3 one  left)
    ;; s2
    (delta s2 zero s2 zero right)
    (delta s2 one  s2 one  right)
    (delta s2 one  s3 zero left)
    ;; s3
    (delta s3 zero  s3 zero  left)
    (delta s3 one   s3 one   left)
    (delta s3 blank s0 blank right)
  )

  (:goal (halted))
)
