
all: man1/ceres.1 man5/ceres.conf.5

man1/ceres.1: ceres.1.md man1 pandoc
	@pandoc -s -t man $< -o $@
	@sed -i.bk 's/\.TH.*/.TH "ceres" "1"/g' man1/ceres.1
	@rm -f man1/ceres.1.bk

man5/ceres.conf.5: ceres.conf.5.md man5 pandoc
	@pandoc -s -t man $< -o $@
	@sed -i.bk 's/\.TH.*/.TH "ceres" "1"/g' man5/ceres.conf.5
	@rm -f man5/ceres.conf.5.bk

man1:
	@mkdir $@

man5:
	@mkdir $@

pandoc:
	@pandoc --version > /dev/null

view: man1/ceres.1 man5/ceres.conf.5
	@man -M . 1 ceres
	@man -M . 5 ceres.conf

clean:
	@-rm -R man1

.PHONY: pandoc view clean

