; same algorithm but with let bindings

(defun elaborate-slow-sort (list)
    (if (or (null list) (null (cdr list)))
        ; zero and one-element lists are already sorted (recursion base case)
        list
        ; longer lists can be sorted by choosing the first element as the pivot,
        ; splitting into lower, equal and higher than the pivot
        ; and sorting and concatenating the individual parts
        (let
            ((lt (remove-if-not (lambda (x) (< x (car list))) list))
                (eq (remove-if-not (lambda (x) (= x (car list))) list))
                (gt (remove-if-not (lambda (x) (> x (car list))) list)))
            (append
                (elaborate-slow-sort lt)
                eq ; elements exactly equal to pivot are already sorted, no need for recursion here
                (elaborate-slow-sort gt)))))
