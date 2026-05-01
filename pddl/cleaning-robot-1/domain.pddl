(define (domain cleaning-robot)
  (:requirements :strips :typing :negative-preconditions)

  (:types room)

  (:predicates
    (robot-at ?x - room)
    (connected ?x ?y - room)
    (unlocked ?x ?y - room)
    (clean ?x - room)
  )

  (:action open
    :parameters (?x ?y - room)
    :precondition (and
      (robot-at ?x)
      (connected ?x ?y)
      (not (unlocked ?x ?y))
    )
    :effect (and (unlocked ?x ?y) (unlocked ?y ?x))
  )

  (:action drive
    :parameters (?x ?y - room)
    :precondition (and
      (robot-at ?x)
      (connected ?x ?y)
      (unlocked ?x ?y)
    )
    :effect (and
      (robot-at ?y)
      (not (robot-at ?x))
    )
  )

  (:action clean
    :parameters (?x - room)
    :precondition (robot-at ?x)
    :effect (clean ?x)
  )
)
