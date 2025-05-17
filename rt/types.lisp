(defparameter *tag-nil* 1)
(defparameter *tag-list* 2)
(defparameter *tag-sint32* 4)
(defparameter *tag-string* 8)
(defparameter *tag-identifier* 16)
(defparameter *tag-function* 32)

(defun listp (thingy)
    ;; nil and cons are lists
    (if thingy (= (intrinsic:type-tag-of thingy) *tag-list*) t))

(defun consp (thingy)
    (= (intrinsic:type-tag-of thingy) *tag-list*))

(defun numberp (thingy)
    (= (intrinsic:type-tag-of thingy) *tag-sint32*))

(defun stringp (thingy)
    (= (intrinsic:type-tag-of thingy) *tag-string*))

(defun symbolp (thingy)
    (= (intrinsic:type-tag-of thingy) *tag-identifier*))

(defun functionp (thingy)
    (= (intrinsic:type-tag-of thingy) *tag-function*))

(defun assert-list (thingy)
    (if (listp thingy) thingy (panic "type error: expected list")))

(defun assert-cons (thingy)
    (if (consp thingy) thingy (panic "type error: expected cons")))

(defun assert-number (thingy)
    (if (numberp thingy) thingy (panic "type error: expected number")))

(defun assert-string (thingy)
    (if (stringp thingy) thingy (panic "type error: expected string")))

(defun assert-symbol (thingy)
    (if (symbolp thingy) thingy (panic "type error: expected symbol")))

(defun assert-function (thingy)
    (if (functionp thingy) thingy (panic "type error: expected function")))
