(defun slow-sort (list)
    (if (or (null list) (null (cdr list)))
        ; zero and one-element lists are already sorted
        list
        ; longer lists can be sorted by splitting into lower, equal and higher,
        ; sorting each part, and then concatenating low, equal and high
        (append
            (slow-sort (remove-if-not (lambda (x) (< x (car list))) list))
            (remove-if-not (lambda (x) (= x (car list))) list)
            (slow-sort (remove-if-not (lambda (x) (> x (car list))) list)))))

(defparameter *list* '(123 2 12322 44 22 33))
(format t "Sorted version of ~a is ~a" *list* (slow-sort *list*))
