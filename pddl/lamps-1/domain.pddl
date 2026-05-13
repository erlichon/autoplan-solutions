(define (domain lamps)
  (:requirements :strips :typing :negative-preconditions :conditional-effects)

  (:types coord)

  (:predicates
    (on ?x ?y - coord)
    (succ ?a ?b - coord)
  )

  (:action flip
    :parameters (?x ?y - coord)
    :effect (and
      ;; Toggle the flipped cell itself.
      (when (on ?x ?y) (not (on ?x ?y)))
      (when (not (on ?x ?y)) (on ?x ?y))

      ;; +x direction: 1st neighbor
      (forall (?x2 - coord)
        (when (and (succ ?x ?x2) (on ?x ?y) (on ?x2 ?y))
          (not (on ?x2 ?y))))
      (forall (?x2 - coord)
        (when (and (succ ?x ?x2) (not (on ?x ?y)) (not (on ?x2 ?y)))
          (on ?x2 ?y)))
      ;; +x direction: 2nd neighbor
      (forall (?x2 ?x3 - coord)
        (when (and (succ ?x ?x2) (succ ?x2 ?x3) (on ?x ?y) (on ?x2 ?y) (on ?x3 ?y))
          (not (on ?x3 ?y))))
      (forall (?x2 ?x3 - coord)
        (when (and (succ ?x ?x2) (succ ?x2 ?x3) (not (on ?x ?y)) (not (on ?x2 ?y)) (not (on ?x3 ?y)))
          (on ?x3 ?y)))

      ;; -x direction: 1st neighbor
      (forall (?x2 - coord)
        (when (and (succ ?x2 ?x) (on ?x ?y) (on ?x2 ?y))
          (not (on ?x2 ?y))))
      (forall (?x2 - coord)
        (when (and (succ ?x2 ?x) (not (on ?x ?y)) (not (on ?x2 ?y)))
          (on ?x2 ?y)))
      ;; -x direction: 2nd neighbor
      (forall (?x2 ?x3 - coord)
        (when (and (succ ?x2 ?x) (succ ?x3 ?x2) (on ?x ?y) (on ?x2 ?y) (on ?x3 ?y))
          (not (on ?x3 ?y))))
      (forall (?x2 ?x3 - coord)
        (when (and (succ ?x2 ?x) (succ ?x3 ?x2) (not (on ?x ?y)) (not (on ?x2 ?y)) (not (on ?x3 ?y)))
          (on ?x3 ?y)))

      ;; +y direction: 1st neighbor
      (forall (?y2 - coord)
        (when (and (succ ?y ?y2) (on ?x ?y) (on ?x ?y2))
          (not (on ?x ?y2))))
      (forall (?y2 - coord)
        (when (and (succ ?y ?y2) (not (on ?x ?y)) (not (on ?x ?y2)))
          (on ?x ?y2)))
      ;; +y direction: 2nd neighbor
      (forall (?y2 ?y3 - coord)
        (when (and (succ ?y ?y2) (succ ?y2 ?y3) (on ?x ?y) (on ?x ?y2) (on ?x ?y3))
          (not (on ?x ?y3))))
      (forall (?y2 ?y3 - coord)
        (when (and (succ ?y ?y2) (succ ?y2 ?y3) (not (on ?x ?y)) (not (on ?x ?y2)) (not (on ?x ?y3)))
          (on ?x ?y3)))

      ;; -y direction: 1st neighbor
      (forall (?y2 - coord)
        (when (and (succ ?y2 ?y) (on ?x ?y) (on ?x ?y2))
          (not (on ?x ?y2))))
      (forall (?y2 - coord)
        (when (and (succ ?y2 ?y) (not (on ?x ?y)) (not (on ?x ?y2)))
          (on ?x ?y2)))
      ;; -y direction: 2nd neighbor
      (forall (?y2 ?y3 - coord)
        (when (and (succ ?y2 ?y) (succ ?y3 ?y2) (on ?x ?y) (on ?x ?y2) (on ?x ?y3))
          (not (on ?x ?y3))))
      (forall (?y2 ?y3 - coord)
        (when (and (succ ?y2 ?y) (succ ?y3 ?y2) (not (on ?x ?y)) (not (on ?x ?y2)) (not (on ?x ?y3)))
          (on ?x ?y3)))
    )
  )
)
