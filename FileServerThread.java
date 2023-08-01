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
        while(!server.hasClose()) {
          if (!server.hasRequest()) continue;
          System.out.println("File server has request!");

          while (!server.hasFilename());
          System.out.println("File server has filename!");

          if (server.filenameExists()) {
            while (!server.eof()) {
              server.sendByte();
            }
            System.out.println("File sent!");
          } else {
            System.out.println("File does not exist!");
          }

          server.sendZeroByte();
        }

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
