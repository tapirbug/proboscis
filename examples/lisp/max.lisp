(defun max-2 (one two)
    (if (> one two) one two))

(defun max-inner (acc rest)
    (if (null rest)
        acc
        (max-inner
            (max-2 acc (car rest))
            (cdr rest))))

(defun maximum (list)
    (max-inner (car list) (cdr list)))

(defparameter *list* '(1 2 3 4))
(format t "Max of ~a is ~a" *list* (maximum *list*))
