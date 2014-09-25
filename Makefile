all: hide-app.exe hide.exe

resource.o: resource.rc resource.h TrayIcon.ico
	windres $< $@

.c.o: resource.h
	cc -Wall -c $<

hide.exe: hide.c
	cc -Wall -o $@ $^ -s

hide-app.exe: hide-app.o resource.o
	cc -o $@ $^ -s -lgdi32 -lcomctl32 -Wl,--subsystem,windows

clean:
	rm *.o *.exe
