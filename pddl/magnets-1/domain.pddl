(define (domain magnets)
  (:requirements :strips :typing :negative-preconditions :conditional-effects)

  (:types magnet coord)

  (:predicates
    (at ?m - magnet ?x ?y - coord)
    (attached ?m ?n - magnet)
    (adj ?x1 ?y1 ?x2 ?y2 - coord)
    (different ?m ?n - magnet)
    (forbidden ?x ?y - coord)
  )

  (:action move
    :parameters (?m - magnet ?x ?y ?xnew ?ynew - coord)
    :precondition (and
      (at ?m ?x ?y)
      (adj ?x ?y ?xnew ?ynew)
      (not (forbidden ?xnew ?ynew))
    )
    :effect (and
      (not (at ?m ?x ?y))
      (at ?m ?xnew ?ynew)
      (forall (?n - magnet)
        (when (attached ?m ?n)
          (and (not (at ?n ?x ?y)) (at ?n ?xnew ?ynew))
        )
      )
    )
  )

  (:action attach
    :parameters (?m ?n - magnet ?x ?y - coord)
    :precondition (and
      (at ?m ?x ?y)
      (at ?n ?x ?y)
      (different ?m ?n)
      (not (attached ?m ?n))
    )
    :effect (and
      (attached ?m ?n)
      (attached ?n ?m)
    )
  )
)
