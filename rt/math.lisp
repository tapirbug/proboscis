(defun + (&rest addends)
    (if (null addends)
        0
        (intrinsic:add-2 (car addends) (apply #'+ (cdr addends)))))

(defun - (arg &rest rest)
    (if (null rest)
        (intrinsic:sub-2 0 arg)
        (subtract-list arg rest)))

;; not a standard function, just lack of module privacy to have this public
(defun subtract-list (subtrahend list)
    (if (null list)
        subtrahend
        (subtract-list (intrinsic:sub-2 subtrahend (car list)) (cdr list))))

;; division would need fractions and is unsupported, but floor can be used

(defun * (&rest factors)
    (if (null factors)
        1
        (intrinsic:mul-2 (car factors) (apply #'* (cdr factors)))))

;; one-argument version not supported
(defun floor (top bottom)
    (intrinsic:div-2 (assert-number top) (assert-number bottom)))

(defun = (first &rest rest)
    (if (null rest)
        t
        (and
            (intrinsic:=-2 first (car rest))
            (apply #'= rest))))

(defun < (first &rest rest)
    (if (null rest)
        t
        (and
            (intrinsic:<-2 first (car rest))
            (apply #'< rest))))

(defun > (first &rest rest)
    (if (null rest)
        t
        (and
            (intrinsic:>-2 first (car rest))
            (apply #'> rest))))

(defun <= (first &rest rest)
    (if (null rest)
        t
        (and
            (intrinsic:<=-2 first (car rest))
            (apply #'<= rest))))

(defun >= (first &rest rest)
    (if (null rest)
        t
        (and
            (intrinsic:>=-2 first (car rest))
            (apply #'>= rest))))

;; the one and three or more argument versions of common lisp are not
;; supported (would need to check all permutations)
(defun /= (first second)
    (intrinsic:/=-2 first second))
