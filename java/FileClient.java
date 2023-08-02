import java.net.*;
import java.io.*;
import jatyc.lib.Typestate;

@Typestate("FileClient")
public class FileClient {
  private Socket socket;
  protected OutputStream out;
  protected BufferedReader in;
  protected int lastByte;

  public boolean start() {
    try {
      socket = new Socket("localhost", 1234);
      out = socket.getOutputStream();
      in = new BufferedReader(new InputStreamReader(socket.getInputStream()));
      return true;
    } catch (Exception e) {
      e.printStackTrace();
      return false;
    }
  }

  public void request(String filename) throws Exception {
    if (filename == null) throw new Exception("Filename cannot be null!");
    out.write("REQUEST\n".getBytes());
    String filenameWithEndline = filename + '\n';
    out.write(filenameWithEndline.getBytes());
  }

  public boolean readByte() throws Exception {
    lastByte = in.read();
    System.out.println("Received byte: " + (char) lastByte);
    return lastByte != 0;
  }

  public void close() throws Exception {
    out.write("CLOSE\n".getBytes());

    socket.close();
    out.close();
    in.close();
  }

  public static void main(String[] args) throws Exception {
    FileClient client = new FileClient();
    if (client.start()) {
      System.out.println("File client started!");
      client.request("test1.txt");
      while (client.readByte());
      client.request("test2.txt");
      while (client.readByte());
      client.request("test3.txt");
      while (client.readByte());
      System.out.println("Request finished!");

      client.close();
    } else {
      System.out.println("Could not start client!");
    }
  }
}
