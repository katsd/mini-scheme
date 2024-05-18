(define (even? x)
  (if (= x 0) #t (odd? (- x 1))))
(define (odd? x)
  (if (= x 0) #f (even? (- x 1))))

(display (even? 100))
(newline)
(display (even? 100000))
