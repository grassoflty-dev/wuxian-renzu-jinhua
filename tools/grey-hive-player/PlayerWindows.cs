using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Wuxian.Player
{
    public static class PlayerWindows
    {
        public sealed class TcpListenerRow
        {
            public readonly string LocalAddress;
            public readonly int LocalPort;
            public readonly uint State, OwningProcess;
            internal TcpListenerRow(string address,int port,uint state,uint owner)
            { LocalAddress=address; LocalPort=port; State=state; OwningProcess=owner; }
        }
        [DllImport("iphlpapi.dll", ExactSpelling=true)]
        private static extern uint GetExtendedTcpTable(IntPtr table, ref uint size,
            [MarshalAs(UnmanagedType.Bool)] bool order, int addressFamily, int tableClass, uint reserved);
        // Only matching-port rows leave this method; other endpoints remain private in the bounded native buffer.
        public static TcpListenerRow[] FindIpv4Listeners(int port)
        {
            if (port<1 || port>65535) throw new ArgumentOutOfRangeException(nameof(port));
            uint size=0; uint status=GetExtendedTcpTable(IntPtr.Zero,ref size,false,2,5,0);
            if (status!=122 || size<4 || size>1048576) throw new Win32Exception((int)status);
            GCHandle pin=default(GCHandle);
            try {
                uint capacity=size;
                byte[] buffer=new byte[checked((int)capacity)];
                pin=GCHandle.Alloc(buffer,GCHandleType.Pinned);
                IntPtr table=pin.AddrOfPinnedObject();
                status=GetExtendedTcpTable(table,ref size,false,2,5,0);
                if (status!=0) throw new Win32Exception((int)status);
                if (size<4 || size>capacity) throw new InvalidDataException("Invalid listener table size");
                uint count=unchecked((uint)Marshal.ReadInt32(table,0));
                const int rowSize=24;
                if (count>(size-4)/rowSize) throw new InvalidDataException("Invalid listener row count");
                var matches=new System.Collections.Generic.List<TcpListenerRow>();
                for (uint i=0;i<count;i++) {
                    int offset=checked(4+checked((int)i*rowSize));
                    int localPort=(Marshal.ReadByte(table,offset+8)<<8)|Marshal.ReadByte(table,offset+9);
                    uint state=unchecked((uint)Marshal.ReadInt32(table,offset));
                    if (localPort!=port || state!=2) continue;
                    string address=string.Format(System.Globalization.CultureInfo.InvariantCulture,"{0}.{1}.{2}.{3}",
                        Marshal.ReadByte(table,offset+4),Marshal.ReadByte(table,offset+5),Marshal.ReadByte(table,offset+6),Marshal.ReadByte(table,offset+7));
                    uint owner=unchecked((uint)Marshal.ReadInt32(table,offset+20));
                    matches.Add(new TcpListenerRow(address,localPort,state,owner));
                }
                return matches.ToArray();
            } finally { if (pin.IsAllocated) pin.Free(); }
        }
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern int WSAStartup(ushort version,IntPtr data);
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern int WSACleanup();
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern UIntPtr WSASocketW(int af,int type,int protocol,IntPtr info,uint group,uint flags);
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern int getsockname(UIntPtr socket,IntPtr name,ref int length);
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern int getsockopt(UIntPtr socket,int level,int option,out int value,ref int length);
        [DllImport("Ws2_32.dll", ExactSpelling=true)] private static extern int closesocket(UIntPtr socket);
        private static void ZeroFree(IntPtr p,int length) { if (p!=IntPtr.Zero) { for (int i=0;i<length;i++) Marshal.WriteByte(p,i,0); Marshal.FreeHGlobal(p); } }
        private static bool VerifyDuplicatedListener(byte[] proof,int port)
        {
            if (proof==null || proof.Length!=628 || port<1 || port>65535) return false;
            IntPtr data=IntPtr.Zero,info=IntPtr.Zero,address=IntPtr.Zero; bool started=false,valid=false;
            UIntPtr socket=UIntPtr.MaxValue;
            try {
                data=Marshal.AllocHGlobal(512); for(int i=0;i<512;i++) Marshal.WriteByte(data,i,0);
                if (WSAStartup(0x0202,data)!=0) return false;
                started=true; if (unchecked((ushort)Marshal.ReadInt16(data))!=0x0202) return false;
                info=Marshal.AllocHGlobal(628); Marshal.Copy(proof,0,info,628);
                socket=WSASocketW(-1,-1,-1,info,0,1);
                if (socket==UIntPtr.MaxValue) return false;
                address=Marshal.AllocHGlobal(16); for(int i=0;i<16;i++) Marshal.WriteByte(address,i,0);
                int length=16;
                if (getsockname(socket,address,ref length)!=0 || length!=16 || Marshal.ReadInt16(address)!=2 ||
                    ((Marshal.ReadByte(address,2)<<8)|Marshal.ReadByte(address,3))!=port ||
                    Marshal.ReadByte(address,4)!=127 || Marshal.ReadByte(address,5)!=0 || Marshal.ReadByte(address,6)!=0 || Marshal.ReadByte(address,7)!=1) return false;
                int optionLength=4, value;
                if (getsockopt(socket,0xffff,2,out value,ref optionLength)!=0 || optionLength!=4 || value==0) return false;
                optionLength=4;
                if (getsockopt(socket,0xffff,0x1008,out value,ref optionLength)!=0 || optionLength!=4 || value!=1) return false;
                optionLength=4;
                if (getsockopt(socket,0xffff,-5,out value,ref optionLength)!=0 || optionLength!=4 || value==0) return false;
                valid=true;
            } finally {
                if (socket!=UIntPtr.MaxValue && closesocket(socket)!=0) valid=false;
                if (started && WSACleanup()!=0) valid=false;
                ZeroFree(address,16); ZeroFree(info,628); ZeroFree(data,512);
            }
            return valid;
        }
        // Continuously drain stderr while retaining only an allowlisted code index.
        // Raw output is never exposed or retained after processing each bounded buffer.
        public sealed class NativeErrorCapture
        {
            private static readonly string[] Codes = new string[] {
                "E_SESSION_ARGS", "E_SESSION_TOKEN", "E_SESSION_NAMESPACE", "E_SESSION_CREATE",
                "E_SESSION_PATH", "E_SESSION_REPARSE", "E_SESSION_ENTRY_REQUIRED",
                "E_READY_CREATE", "E_READY_OCCUPIED", "E_READY_PATH", "E_READY_SERIALIZE", "E_READY_WRITE",
                "E_SOCKET_BIND", "E_SOCKET", "E_SOCKET_ACCEPT", "E_RPC_CORE", "E_OWNER_PROOF"
            };
            public sealed class StartupSnapshot
            {
                public readonly bool Seen, Malformed;
                public readonly int State, Count;
                public readonly long NativeSampleUnixMs, ReceiveUnixMs;
                internal StartupSnapshot(bool seen,int state,long nativeMs,long receiveMs,int count,bool malformed)
                { Seen=seen; State=state; NativeSampleUnixMs=nativeMs; ReceiveUnixMs=receiveMs; Count=count; Malformed=malformed; }
            }
            private readonly object proofLock = new object();
            private static readonly byte[] ProofPrefix=System.Text.Encoding.ASCII.GetBytes("GH_OWNER_SOCKET_V1:");
            private readonly byte[] proofLine=new byte[19+1256];
            private int proofLength=0; private bool proofOverflow=false,proofSeen=false,proofInvalid=false,proofConsumed=false;
            private byte[] proofBytes=null; private int verifiedPort=0;
            private static int Hex(byte b) { return b>=48 && b<=57 ? b-48 : b>=97 && b<=102 ? b-87 : -1; }
            private void CaptureProofByte(byte value) {
                lock(proofLock) {
                    if (value!=10) { if (proofLength<proofLine.Length) proofLine[proofLength++]=value; else proofOverflow=true; return; }
                    bool prefix=proofLength>=ProofPrefix.Length;
                    for (int i=0;prefix && i<ProofPrefix.Length;i++) if (proofLine[i]!=ProofPrefix[i]) prefix=false;
                    if (prefix) {
                        if (proofSeen || proofConsumed || proofOverflow || proofLength!=ProofPrefix.Length+1256) proofInvalid=true;
                        else {
                            byte[] parsed=new byte[628]; bool valid=true;
                            for (int i=0;i<628;i++) { int hi=Hex(proofLine[ProofPrefix.Length+2*i]),lo=Hex(proofLine[ProofPrefix.Length+2*i+1]); if (hi<0 || lo<0) { valid=false; break; } parsed[i]=(byte)((hi<<4)|lo); }
                            if (valid) { proofSeen=true; proofBytes=parsed; } else { proofInvalid=true; Array.Clear(parsed,0,parsed.Length); }
                        }
                    }
                    Array.Clear(proofLine,0,proofLine.Length); proofLength=0; proofOverflow=false;
                }
            }
            // 0 means pending, 1 is a cached verified port, -1 is fail-closed. Raw proof never leaves this object.
            public int VerifyOwnerProof(int port) {
                lock(proofLock) {
                    if (proofInvalid || port<1 || port>65535) return -1;
                    if (proofConsumed) return verifiedPort==port ? 1 : -1;
                    if (!proofSeen || proofBytes==null) return 0;
                    proofConsumed=true;
                    try { if (!VerifyDuplicatedListener(proofBytes,port)) { proofInvalid=true; return -1; } verifiedPort=port; return 1; }
                    finally { Array.Clear(proofBytes,0,proofBytes.Length); proofBytes=null; }
                }
            }
            private readonly object startupLock = new object();
            private readonly byte[] startupLine = new byte[40];
            private int startupLength=0, startupCount=0, startupState=-1;
            private long startupNativeMs=0, startupReceiveMs=0;
            private bool startupOverflow=false, startupSeen=false, startupMalformed=false;
            public StartupSnapshot GetStartupSnapshot() {
                lock (startupLock) {
                    bool partial=startupLength>=6 && startupLine[0]==71 && startupLine[1]==72 && startupLine[2]==95 && startupLine[3]==83 && startupLine[4]==67 && startupLine[5]==95;
                    return new StartupSnapshot(startupSeen,startupState,startupNativeMs,startupReceiveMs,startupCount,startupMalformed || partial);
                }
            }
            private void CaptureStartupByte(byte value) {
                lock (startupLock) {
                    if (value!=10) { if (startupLength<startupLine.Length) startupLine[startupLength++]=value; else startupOverflow=true; return; }
                    bool prefix=startupLength>=6 && startupLine[0]==71 && startupLine[1]==72 && startupLine[2]==95 && startupLine[3]==83 && startupLine[4]==67 && startupLine[5]==95;
                    if (prefix) {
                        bool valid=!startupOverflow && startupLength>=9 && startupLength<=21 && startupLine[7]==58 && (startupLine[6]==48 || startupLine[6]==49 || startupLine[6]==85);
                        long stamp=0;
                        if (valid) for (int i=8;i<startupLength;i++) { int digit=startupLine[i]-48; if (digit<0 || digit>9 || stamp>(9999999999999L-digit)/10) { valid=false; break; } stamp=stamp*10+digit; }
                        if (stamp<=0 || startupCount>=96) valid=false;
                        if (valid) { startupSeen=true; startupState=startupLine[6]==48?0:startupLine[6]==49?1:-1; startupNativeMs=stamp; startupReceiveMs=DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(); startupCount++; }
                        else startupMalformed=true;
                    }
                    Array.Clear(startupLine,0,startupLine.Length); startupLength=0; startupOverflow=false;
                }
            }
            private int codeIndex = -1;
            private int outputSeen = 0;
            public bool HasOutput { get { return System.Threading.Volatile.Read(ref outputSeen) != 0; } }
            public readonly System.Threading.Tasks.Task Completion;
            public string SafeCode { get { int n = System.Threading.Volatile.Read(ref codeIndex); return n >= 0 && n < Codes.Length ? Codes[n] : "UNKNOWN"; } }
            public NativeErrorCapture(Stream input)
            {
                if (input == null) throw new ArgumentNullException(nameof(input));
                Completion = Drain(input);
            }
            private async System.Threading.Tasks.Task Drain(Stream input)
            {
                byte[] buffer = new byte[512];
                ulong mask = (1UL << Codes.Length) - 1UL;
                int position = 0; bool carriage = false;
                try {
                    int count;
                    while ((count = await input.ReadAsync(buffer, 0, buffer.Length).ConfigureAwait(false)) != 0) {
                        System.Threading.Volatile.Write(ref outputSeen, 1);
                        for (int b = 0; b < count; b++) {
                            byte value = buffer[b];
                            CaptureStartupByte(value); CaptureProofByte(value);
                            if (value == 10) {
                                for (int i = 0; i < Codes.Length; i++) {
                                    if ((mask & (1UL << i)) != 0 && position == Codes[i].Length) System.Threading.Volatile.Write(ref codeIndex, i);
                                }
                                mask = (1UL << Codes.Length) - 1UL; position = 0; carriage = false;
                            } else if (value == 13 && !carriage) {
                                carriage = true;
                            } else {
                                if (carriage) mask = 0;
                                for (int i = 0; i < Codes.Length; i++) {
                                    if ((mask & (1UL << i)) != 0 && (position >= Codes[i].Length || value != (byte)Codes[i][position])) mask &= ~(1UL << i);
                                }
                                if (position < 128) position++;
                            }
                        }
                        Array.Clear(buffer, 0, count);
                    }
                } finally { Array.Clear(buffer, 0, buffer.Length); lock(proofLock) { Array.Clear(proofLine,0,proofLine.Length); if (proofBytes!=null) { Array.Clear(proofBytes,0,proofBytes.Length); proofBytes=null; } if (!proofConsumed) proofInvalid=true; } }
            }
        }
        public sealed class FileIdentity
        {
            public readonly ulong VolumeSerialNumber, FileIdLow, FileIdHigh;
            public readonly long CreationTimeUtcTicks, Length;
            public readonly uint Attributes;
            internal FileIdentity(ulong volume, ulong low, ulong high, long creation, long length, uint attributes)
            {
                VolumeSerialNumber = volume; FileIdLow = low; FileIdHigh = high;
                CreationTimeUtcTicks = creation; Length = length; Attributes = attributes;
            }
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct FileTime { public uint Low, High; }
        [StructLayout(LayoutKind.Sequential)]
        private struct HandleInformation
        {
            public uint Attributes;
            public FileTime CreationTime, LastAccessTime, LastWriteTime;
            public uint VolumeSerialNumber, SizeHigh, SizeLow, NumberOfLinks, IndexHigh, IndexLow;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct FileIdInformation { public ulong VolumeSerialNumber, Low, High; }
        [StructLayout(LayoutKind.Sequential)]
        private struct DispositionInformation { public byte DeleteFile; }
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CreateDirectoryW(string path, IntPtr securityAttributes);
        [DllImport("shell32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
        private static extern IntPtr CommandLineToArgvW(string commandLine, out int count);
        [DllImport("kernel32.dll", ExactSpelling = true)]
        private static extern IntPtr LocalFree(IntPtr memory);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
        private static extern SafeFileHandle CreateFileW(string path, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);
        [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
        private static extern uint GetFileType(SafeFileHandle handle);
        [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetFileInformationByHandle(SafeFileHandle handle, out HandleInformation information);
        [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetFileInformationByHandleEx(SafeFileHandle handle, int informationClass, out FileIdInformation information, uint size);
        [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetFileInformationByHandle(SafeFileHandle handle, int informationClass, ref DispositionInformation information, uint size);

        public static void CreateDirectoryNew(string path)
        {
            if (String.IsNullOrEmpty(path) || path.IndexOf('\0') >= 0) throw new ArgumentException("Invalid directory path.");
            if (!CreateDirectoryW(path, IntPtr.Zero)) throw new Win32Exception(Marshal.GetLastWin32Error());
        }
        public static string[] SplitArguments(string commandLine)
        {
            if (String.IsNullOrEmpty(commandLine) || commandLine.IndexOf('\0') >= 0) throw new ArgumentException("Invalid command line.");
            int count;
            IntPtr memory = CommandLineToArgvW(commandLine, out count);
            if (memory == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            try
            {
                if (count <= 0) throw new IOException("Invalid argument count.");
                string[] result = new string[count];
                for (int i = 0; i < count; i++) result[i] = Marshal.PtrToStringUni(Marshal.ReadIntPtr(memory, checked(i * IntPtr.Size)));
                return result;
            }
            finally { LocalFree(memory); }
        }
        // Caller owns the returned pin; only read sharing is permitted, never write/delete sharing.
        public static SafeFileHandle OpenReadPinned(string path, bool deleteAccess)
        {
            if (String.IsNullOrEmpty(path) || path.IndexOf('\0') >= 0) throw new ArgumentException("Invalid file path.");
            SafeFileHandle handle = CreateFileW(path, 0x80000000u | (deleteAccess ? 0x00010000u : 0u), 1u, IntPtr.Zero, 3u, 0x00200000u, IntPtr.Zero);
            if (handle.IsInvalid) { int error = Marshal.GetLastWin32Error(); handle.Dispose(); throw new Win32Exception(error); }
            try { ReadIdentity(handle); return handle; }
            catch { handle.Dispose(); throw; }
        }
        // Borrows the handle. Identity uses FILE_ID_INFO, never the legacy 64-bit file index.
        public static FileIdentity ReadIdentity(SafeFileHandle handle)
        {
            if (handle == null || handle.IsInvalid || handle.IsClosed) throw new ArgumentException("Invalid file handle.");
            if (GetFileType(handle) != 1u) throw new IOException("An ordinary disk file is required.");
            HandleInformation info;
            if (!GetFileInformationByHandle(handle, out info)) throw new Win32Exception(Marshal.GetLastWin32Error());
            if ((info.Attributes & 0x410u) != 0u) throw new IOException("Directory and reparse handles are forbidden.");
            FileIdInformation id;
            if (!GetFileInformationByHandleEx(handle, 18, out id, (uint)Marshal.SizeOf(typeof(FileIdInformation)))) throw new Win32Exception(Marshal.GetLastWin32Error());
            long creation = checked((long)(((ulong)info.CreationTime.High << 32) | info.CreationTime.Low));
            long length = checked((long)(((ulong)info.SizeHigh << 32) | info.SizeLow));
            return new FileIdentity(id.VolumeSerialNumber, id.Low, id.High, DateTime.FromFileTimeUtc(creation).Ticks, length, info.Attributes);
        }
        // Consumes the handle even on failure; the caller must verify captured identity/hash first.
        public static void DeletePinned(SafeFileHandle handle)
        {
            if (handle == null) throw new ArgumentNullException("handle");
            try
            {
                ReadIdentity(handle);
                DispositionInformation disposition = new DispositionInformation { DeleteFile = 1 };
                if (!SetFileInformationByHandle(handle, 4, ref disposition, (uint)Marshal.SizeOf(typeof(DispositionInformation)))) throw new Win32Exception(Marshal.GetLastWin32Error());
            }
            finally { handle.Dispose(); }
        }
    }
}
