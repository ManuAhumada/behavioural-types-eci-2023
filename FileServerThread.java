import java.net.*;

public class FileServerThread extends Thread {
  private Socket socket;

  public FileServerThread(Socket socket) {
    this.socket = socket;
  }

  public void run() {
    try {
      FileServer server = new FileServer();
      if (server.start(socket)) {
        System.out.println("File server started!");
        do {
          if (!server.hasRequest()) continue;

          while (!server.hasFilename());

          if (server.filenameExists()) {
            while (!server.eof()) {
              server.sendByte();
            }
          }

          server.sendZeroByte();
        } while(!server.hasClose());

        server.close();
        System.out.println("File server closed!");
      } else {
        System.out.println("Could not start server!");
      }
    } catch (Exception e) {
      e.printStackTrace();
    }
  }
}
