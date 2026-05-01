(define (domain turing-machine)
  (:requirements :strips :typing :negative-preconditions)

  (:types position state symbol direction)

  (:predicates
    (head ?p - position)
    (current-state ?s - state)
    (tape ?p - position ?x - symbol)
    (delta ?s - state ?x - symbol ?s2 - state ?x2 - symbol ?d - direction)
    (next ?p - position ?p2 - position ?d - direction)
    (accepting ?s - state)
    (is-blank ?x - symbol)
    (halted)
  )

  (:action transition
    :parameters (?p ?p2 - position ?s ?s2 - state ?x ?x2 - symbol ?d - direction)
    :precondition (and
      (head ?p)
      (current-state ?s)
      (tape ?p ?x)
      (delta ?s ?x ?s2 ?x2 ?d)
      (next ?p ?p2 ?d)
      (not (halted))
    )
    :effect (and
      (not (head ?p))
      (head ?p2)
      (not (current-state ?s))
      (current-state ?s2)
      (not (tape ?p ?x))
      (tape ?p ?x2)
    )
  )

  (:action halt
    :parameters (?p - position ?s - state ?x - symbol)
    :precondition (and
      (head ?p)
      (current-state ?s)
      (tape ?p ?x)
      (accepting ?s)
      (is-blank ?x)
      (not (halted))
    )
    :effect (halted)
  )
)
