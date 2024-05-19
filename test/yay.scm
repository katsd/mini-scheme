;(define (even? x)
;  (if (= x 0) #t (odd? (- x 1))))
;(define (odd? x)
;  (if (= x 0) #f (even? (- x 1))))
;
;(display (even? 100))
;(newline)
;(display (even? 100000))

(define-syntax when
  (syntax-rules ()
    ((_ test expr)
      (if test expr))

    ((_ test expr1 expr2 ...)
      (if test
        (begin
          expr1
          expr2 ...)))))

(when (> 1 0)
  (display "yay")
  (newline))

(when (< 1 3) (display "yayyay"))