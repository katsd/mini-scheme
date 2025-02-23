(define (list . l)
  (if (null? l)
    '()
    (cons (car l)
      (apply list (cdr l)))))

(define (length l)
  (if (null? l)
    0
    (+ 1 (length (cdr l)))))

(define (memq a l)
  (cond
    ((null? l) #f)
    ((eq? a (car l)) l)
    (else (memq a (cdr l)))))

(define (last l)
  (if (pair? (cdr l))
    (last (cdr l))
    (car l)))

(define (append . lists)
  (if (null? lists)
    '()
    (if (null? (car lists))
      (apply append (cdr lists))
      (cons (car (car lists)) (apply append (cdr (car lists)) (cdr lists))))))

(define (neq? l r) (not (eq? l r)))

(define (newline) (display "\n"))

(define (+ . a)
  (if (null? a)
    0
    (~+ (car a) (apply + (cdr a)))))

(define (- . a)
  (cond
    ((null? a) 0)
    ((null? (cdr a)) (~- 0 (car a)))
    (else (~- (car a) (apply + (cdr a))))))

(define (* . a)
  (if (null? a)
    1
    (~* (car a) (apply * (cdr a)))))

(define (/ . a)
  (cond
    ((null? a) '())
    ((null? (cdr a)) (~/ 1 (car a)))
    (else (~/ (car a) (apply * (cdr a))))))

(define (= . a)
  (if (~> 2 (length a))
    '()
    (and
      (~= (car a) (car (cdr a)))
      (if (~= 1 (length (cdr a)))
        #t
        (apply = (cdr a))))))

(define (< . a)
  (if (~> 2 (length a))
    '()
    (and
      (~< (car a) (car (cdr a)))
      (if (~= 1 (length (cdr a)))
        #t
        (apply < (cdr a))))))

(define (<= . a)
  (if (~> 2 (length a))
    '()
    (and
      (~<= (car a) (car (cdr a)))
      (if (~= 1 (length (cdr a)))
        #t
        (apply <= (cdr a))))))

(define (> . a)
  (if (~> 2 (length a))
    '()
    (and
      (~> (car a) (car (cdr a)))
      (if (~= 1 (length (cdr a)))
        #t
        (apply > (cdr a))))))

(define (>= . a)
  (if (~> 2 (length a))
    '()
    (and
      (~>= (car a) (car (cdr a)))
      (if (~= 1 (length (cdr a)))
        #t
        (apply >= (cdr a))))))

(define (string-append . a)
  (if (null? a)
    ""
    (~string-append (car a) (apply string-append (cdr a)))))
