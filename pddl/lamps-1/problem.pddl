(define (problem lamps-1)
  (:domain lamps)

  (:objects
    l0 l1 l2 - coord
  )

  (:init
    (on l0 l0)
    (on l1 l0)
    (on l2 l0)
    (on l0 l1)
    (on l0 l2)

    (succ l0 l1)
    (succ l1 l2)
  )

  (:goal (and
    (not (on l0 l0))
    (not (on l1 l0))
    (not (on l2 l0))
    (not (on l0 l1))
    (not (on l1 l1))
    (not (on l2 l1))
    (not (on l0 l2))
    (not (on l1 l2))
    (not (on l2 l2))
  ))
)
