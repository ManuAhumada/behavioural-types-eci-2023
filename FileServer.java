import java.net.*;
import java.io.*;
import jatyc.lib.Typestate;

@Typestate("FileServer")
public class FileServer {
  private Socket socket;
  protected OutputStream out;
  protected BufferedReader in;
  protected String lastFilename;
  private FileReader fileReader;
  private int lastByte;

  public boolean start(Socket s) {
    try {
      socket = s;
      out = socket.getOutputStream();
      in = new BufferedReader(new InputStreamReader(socket.getInputStream()));
      return true;
    } catch (Exception e) {
      e.printStackTrace();
      return false;
    }
  }

  public boolean hasRequest() throws Exception {
    String command = in.readLine();
    return command != null && command.equals("REQUEST");
  }

  public boolean hasClose() throws Exception {
    String command = in.readLine();
    return command != null && command.equals("CLOSE");
  }

  public boolean hasFilename() throws Exception {
    String filename = in.readLine();
    if (filename == null) return false;

    lastFilename = filename;
    return true;
  }

  public boolean filenameExists() {
    File f = new File(lastFilename);

    try {
      fileReader = new FileReader(f);
      return true;
    } catch (Exception e) {
      return false;
    }
  }

  public boolean eof() throws Exception {
    lastByte = fileReader.read();
    if (lastByte != -1) return false;

    fileReader.close();
    return true;
  }

  public void sendByte() throws Exception {
    System.out.println("Sending byte: " + (char) lastByte);
    out.write(lastByte);
  }

  public void sendZeroByte() throws Exception {
    System.out.println();
    out.write(0);
  }

  public void close() throws Exception {
    in.close();
    out.close();
  }

  public static void main(String[] args) throws Exception {
    ServerSocket serverSocket = new ServerSocket(1234);
    while (true) {
      new FileServerThread(serverSocket.accept()).start();
    }
  }
}
