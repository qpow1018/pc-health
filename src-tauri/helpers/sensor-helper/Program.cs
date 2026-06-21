using PcHealth.SensorHelper;

using var sampler = new LibreHardwareCpuSampler();
var host = new HelperHost(sampler, Environment.ProcessId);
host.Run(Console.In, Console.Out, Console.Error);
