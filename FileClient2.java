import java.net.*;
import java.io.*;
import jatyc.lib.Typestate;

@Typestate("FileClient2")
public class FileClient2 extends FileClient {

  public boolean readLine() throws Exception{
    lastByte = in.read();
    String line = "";
    while (lastByte != 0 && (char)lastByte != '\n'){
      line += (char)lastByte;
      lastByte = in.read();
    }
    System.out.println("Received line: " + line);
    return lastByte != 0;
  }

  public static void main(String[] args) throws Exception {
    FileClient2 client = new FileClient2();
    if (client.start()) {
      System.out.println("File client started!");
      client.request("test1.txt");
      while (client.readLine());
      client.request("test2.txt");
      while (client.readLine());
      client.request("test3.txt");
      while (client.readLine());
      System.out.println("Request finished!");

      client.close();
    } else {
      System.out.println("Could not start client!");
    }
  }
}
