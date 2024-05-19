;(define (even? x)
;  (if (= x 0) #t (odd? (- x 1))))
;(define (odd? x)
;  (if (= x 0) #f (even? (- x 1))))
;
;(display (even? 100))
;(newline)
;(display (even? 100000))

;(define-syntax when
;  (syntax-rules ()
;    ((_ test expr)
;      (if test expr))
;
;    ((_ test expr1 expr2 ...)
;      (if test
;        (begin
;          expr1
;          expr2 ...)))))
;
;(when (> 1 0)
;  (display "yay")
;  (newline))
;
;(when (< 1 3) (display "yayyay"))
;
;

;(define-syntax li
;  (syntax-rules ()
;    ((_) '())
;    ((_ a) (cons a (li)))
;    ((_ a b ...) (cons a (li b ...)))))
;
;(display (li))
;(newline)
;
;
;(display (li 1))
;(newline)
;
;
;(display (li 1 2 3 4 5 6))
;(newline)

(define-syntax swap
  (syntax-rules ()
    ((_ x y)
      (let ((a y))
        (set! y x)
        (set! x a)))))

(define a 123)
(define b 456)

(display (string-append "a: " (number->string a)))
(newline)
(display (string-append "b: " (number->string b)))
(newline)

(display "----")
(newline)

(swap a b)

(display (string-append "a: " (number->string a)))
(newline)
(display (string-append "b: " (number->string b)))
(newline)
