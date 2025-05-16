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
