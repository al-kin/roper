(in-package :ropush)

;; some dynamic defs, to prevent unbound variable errors
(defvar $$push nil)
(defvar $$pop)
(defvar $$peek)
(defvar $$depth)
(defvar $$stack-of)
;(defun $pop ())
;(defun $push ())
;(defun $depth ())
;(defun $peek ())
(defvar $stacks)
(defvar $counter)
(defvar $unicorn)
(defvar $halt)
(defvar $types)

(deftype bytes () '(vector (unsigned-byte 8)))

(defun debugging (&optional on)
  (if (and (not on) (member :debugging *features*))
      (progn
	(format t "[-] DEBUGGING OFF~%")
	(setq *features* (remove :debugging *features*)))
      (progn
	(format t "[+] DEBUGGING ON~%")
	(if (not (member :debugging *features*))
	    (push :debugging *features*)))))

(debugging t)

(export '*operations*)
(defvar *operations* ())

;; the list provided in ropush-params is meant to be editable.
;; this list furnishes the defaults.
(export '*stack-types*)
(defparameter *stack-types* '(:input!
			      :output!
			      :exec
			      :code
			      :womb
			      :gadget
			      :bool
			      :int
			      :ratio
			      :bytes))

(export '*immutable-stacks*)
(defparameter *immutable-stacks* '(:input!))

(defparameter *untyped-stacks* '(:exec
				 :code
				 :womb))

;;;;
;; inspired by spector's autoconstructive systems (pushpop, etc),
;; the :womb stack can be used as both a secondary exec stack, to
;; facilitate the rearrangement of instructions, and also as a
;; reproductive organ. some potential reproductive strategies:
;; * clone original exec, pre-run, with mutations
;; * clone womb stack
;; * crossover exec x exec
;; * crossover womb x womb
;; * crossover womb x exec
;; The last seems like the most fruitful, a fortiori. Simple womb
;; cloning could risk having creature sizes dwindle, or explode,
;; depending on the early frequency of dup-like instructions. 
