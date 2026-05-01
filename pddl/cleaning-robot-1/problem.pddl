(define (problem cleaning-robot-1)
  (:domain cleaning-robot)

  (:objects
    a b c d - room
  )

  (:init
      (robot-at a)

      (connected a b)
      (connected b a)
      (connected b c)
      (connected c b)
      (connected b d)
      (connected d b)
  )

  (:goal (and
    (clean b)
    (clean c)
  ))
)
